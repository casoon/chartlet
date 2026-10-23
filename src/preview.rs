//! `chartlet render --preview`: a page on the loopback interface that shows the latest render with
//! its warnings and reloads itself after the next one. Part of the command-line program, not of
//! the library; nothing of it reaches the rendered output.

use std::{
    fmt::Write as _,
    io::{self, Read, Write},
    net::{Ipv4Addr, TcpListener, TcpStream},
    sync::{Arc, Mutex, PoisonError},
    thread,
    time::Duration,
};

use chartlet::{ChartWarning, Styles, stylesheet};

use crate::{Failure, Job};

/// The preview of one watched specification.
pub struct Preview {
    /// The name of the specification file, for the page title.
    name: String,
    /// The shared stylesheet, which the page carries when charts are rendered without theirs.
    stylesheet: Option<String>,
    port: u16,
    state: Mutex<State>,
}

#[derive(Default)]
struct State {
    /// Counts the renders; the page reloads when it changes.
    version: u64,
    /// The last output that rendered.
    chart: Option<String>,
    /// The failure of the latest render, after which `chart` is stale.
    error: Option<String>,
    warnings: Vec<String>,
}

impl Preview {
    /// Listens on `port` of 127.0.0.1, or on a free port for 0, and answers requests on a thread
    /// of its own until the process ends. Prints the address to stdout.
    pub fn serve(port: u16, job: &Job) -> Result<Arc<Self>, Failure> {
        let listener = TcpListener::bind((Ipv4Addr::LOCALHOST, port))
            .and_then(|listener| Ok((listener.local_addr()?.port(), listener)))
            .map_err(|error| format!("could not serve the preview on 127.0.0.1:{port}: {error}"));
        let (port, listener) = listener?;
        let preview = Arc::new(Self {
            name: job.input.file_name().map_or_else(
                || job.input.display().to_string(),
                |name| name.to_string_lossy().into_owned(),
            ),
            stylesheet: (job.options.styles == Styles::External).then(stylesheet),
            port,
            state: Mutex::new(State::default()),
        });
        let server = Arc::clone(&preview);
        thread::spawn(move || {
            for stream in listener.incoming().flatten() {
                // A client that goes away mid-request concerns only that request.
                let _ = server.answer(stream);
            }
        });
        println!("chartlet: preview at http://127.0.0.1:{port}/");
        Ok(preview)
    }

    /// Records a render: the output, or the failure that leaves the previous output stale.
    pub fn update(&self, result: Result<String, String>, warnings: &[ChartWarning]) {
        let mut state = self.state.lock().unwrap_or_else(PoisonError::into_inner);
        state.version += 1;
        state.warnings = warnings
            .iter()
            .map(|warning| {
                format!(
                    "warning[{}] at {}: {}",
                    warning.code, warning.path, warning.message
                )
            })
            .collect();
        match result {
            Ok(chart) => {
                state.chart = Some(chart);
                state.error = None;
            }
            Err(error) => state.error = Some(error),
        }
    }

    /// Answers one request: the page at `/`, the render count at `/version`, nothing else. A
    /// request that names another host is refused, so that no other site can read the preview
    /// through a name that resolves to the loopback address.
    fn answer(&self, mut stream: TcpStream) -> io::Result<()> {
        stream.set_read_timeout(Some(Duration::from_secs(2)))?;
        let mut request = [0; 8192];
        let mut length = 0;
        while length < request.len() && !request[..length].windows(4).any(|end| end == b"\r\n\r\n")
        {
            match stream.read(&mut request[length..])? {
                0 => break,
                read => length += read,
            }
        }
        let request = String::from_utf8_lossy(&request[..length]);
        let mut lines = request.lines();
        let mut start = lines.next().unwrap_or_default().split(' ');
        let (method, target) = (start.next(), start.next());
        let host = lines.find_map(|line| {
            let (name, value) = line.split_once(':')?;
            name.eq_ignore_ascii_case("host").then(|| value.trim())
        });
        let local = host.is_some_and(|host| {
            host == format!("127.0.0.1:{}", self.port) || host == format!("localhost:{}", self.port)
        });
        let (status, content_type, body) = match (method, target) {
            _ if !local => (
                "403 Forbidden",
                "text/plain",
                "not a local request".to_owned(),
            ),
            (Some("GET"), Some("/")) => ("200 OK", "text/html; charset=utf-8", self.page()),
            (Some("GET"), Some("/version")) => ("200 OK", "text/plain", self.version().to_string()),
            (Some("GET"), _) => ("404 Not Found", "text/plain", "not found".to_owned()),
            _ => (
                "405 Method Not Allowed",
                "text/plain",
                "only GET".to_owned(),
            ),
        };
        write!(
            stream,
            "HTTP/1.1 {status}\r\nContent-Type: {content_type}\r\nContent-Length: {}\r\n\
             Cache-Control: no-store\r\nX-Content-Type-Options: nosniff\r\n\
             Content-Security-Policy: default-src 'none'; style-src 'unsafe-inline'; \
             script-src 'unsafe-inline'; connect-src 'self'; img-src data:\r\n\
             Connection: close\r\n\r\n{body}",
            body.len()
        )?;
        stream.flush()
    }

    fn version(&self) -> u64 {
        self.state
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .version
    }

    /// The page: the state of the latest render, its warnings and the chart.
    fn page(&self) -> String {
        let state = self.state.lock().unwrap_or_else(PoisonError::into_inner);
        let mut status = String::new();
        match (&state.error, &state.chart) {
            (None, Some(_)) => {
                let _ = write!(
                    status,
                    "<p class=\"status ok\" role=\"status\">Rendered, {} warning(s).</p>",
                    state.warnings.len()
                );
            }
            (None, None) => status.push_str("<p class=\"status\" role=\"status\">Rendering…</p>"),
            (Some(error), chart) => {
                let _ = write!(
                    status,
                    "<p class=\"status error\" role=\"alert\"><strong>Error:</strong> {}<br>{}</p>",
                    escape(error),
                    if chart.is_some() {
                        "Stale: the chart below is the last output that rendered."
                    } else {
                        "Nothing has rendered yet."
                    }
                );
            }
        }
        for warning in &state.warnings {
            let _ = write!(status, "<p class=\"warning\">{}</p>", escape(warning));
        }
        format!(
            "<!doctype html>
<html lang=\"en\">
<head>
<meta charset=\"utf-8\">
<meta name=\"viewport\" content=\"width=device-width, initial-scale=1\">
<title>{name} – chartlet preview</title>
<style>
body {{ margin: 0; padding: 16px; font: 15px/1.45 system-ui, sans-serif; }}
.status, .warning {{ margin: 0 0 8px; padding: 6px 10px; border-radius: 4px; background: #e4e4e7; color: #18181b; }}
.ok {{ background: #dcfce7; }}
.error {{ background: #fee2e2; }}
.warning {{ background: #fef9c3; font-family: ui-monospace, monospace; font-size: 13px; }}
main {{ margin-top: 16px; }}
main.stale {{ opacity: .4; outline: 3px dashed #dc2626; outline-offset: 4px; }}
</style>
{stylesheet}</head>
<body>
<header>{status}</header>
<main{stale}>{chart}</main>
<script>
setInterval(async () => {{
  try {{
    const response = await fetch('/version', {{ cache: 'no-store' }});
    if ((await response.text()) !== '{version}') location.reload();
  }} catch {{}}
}}, 500);
</script>
</body>
</html>
",
            name = escape(&self.name),
            stylesheet = self
                .stylesheet
                .as_ref()
                .map_or_else(String::new, |css| format!("<style>{css}</style>\n")),
            stale = if state.error.is_some() && state.chart.is_some() {
                " class=\"stale\""
            } else {
                ""
            },
            chart = state.chart.as_deref().unwrap_or_default(),
            version = state.version,
        )
    }
}

fn escape(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}
