use chartlet::qr::{DataTooLong, ErrorCorrection, QrCode};

fn rows(code: &QrCode) -> Vec<String> {
    (0..code.size())
        .map(|y| {
            (0..code.size())
                .map(|x| if code.is_dark(x, y) { '#' } else { '.' })
                .collect()
        })
        .collect()
}

#[test]
fn picks_the_smallest_version_that_fits() {
    // Version 1 holds 17 bytes at level Low and 7 at level High.
    let version = |text: &str, level| QrCode::encode(text, level).unwrap().version();
    assert_eq!(version("", ErrorCorrection::Low), 1);
    assert_eq!(version(&"a".repeat(17), ErrorCorrection::Low), 1);
    assert_eq!(version(&"a".repeat(18), ErrorCorrection::Low), 2);
    assert_eq!(version(&"a".repeat(7), ErrorCorrection::High), 1);
    assert_eq!(version(&"a".repeat(8), ErrorCorrection::High), 2);
}

#[test]
fn version_40_is_the_limit() {
    let largest = QrCode::encode(&"a".repeat(2953), ErrorCorrection::Low).unwrap();
    assert_eq!(largest.version(), 40);
    assert_eq!(largest.size(), 177);
    assert_eq!(
        QrCode::encode(&"a".repeat(2954), ErrorCorrection::Low),
        Err(DataTooLong {
            bytes: 2954,
            max_bytes: 2953
        })
    );
}

#[test]
fn matches_a_code_checked_with_a_reader() {
    let code = QrCode::encode("HELLO WORLD", ErrorCorrection::Quartile).unwrap();
    assert_eq!(code.version(), 1);
    assert_eq!(
        rows(&code),
        [
            "#######.#..#..#######",
            "#.....#..#....#.....#",
            "#.###.#.#..#..#.###.#",
            "#.###.#.#.##..#.###.#",
            "#.###.#..##.#.#.###.#",
            "#.....#.##.#..#.....#",
            "#######.#.#.#.#######",
            "........#.###........",
            ".#.#.####..#####.##.#",
            "..####...#....##...#.",
            ".#..#.##.#.##..#.##.#",
            "#.###..#.####.#.##.##",
            ".#.##.#.#.##.####.#..",
            "........##..#...#.#..",
            "#######.##.#..######.",
            "#.....#.#####..#....#",
            "#.###.#..#..###...##.",
            "#.###.#.#.#....######",
            "#.###.#...#.#.#.#.#.#",
            "#.....#.#.##.#.......",
            "#######...#.#..#.###.",
        ]
    );
}

#[test]
fn function_patterns_are_in_place_for_large_versions() {
    let code = QrCode::encode(&"x".repeat(400), ErrorCorrection::Medium).unwrap();
    let size = code.size();
    assert_eq!(size, 17 + 4 * usize::from(code.version()));
    // Finder pattern corners and their light separators.
    for (x, y) in [(0, 0), (size - 7, 0), (0, size - 7)] {
        assert!(code.is_dark(x, y) && code.is_dark(x + 6, y + 6));
        assert!(!code.is_dark(x + 1, y + 1));
    }
    assert!(!code.is_dark(7, 7));
    // Timing pattern between the finders.
    for i in 8..size - 8 {
        assert_eq!(code.is_dark(i, 6), i % 2 == 0);
        assert_eq!(code.is_dark(6, i), i % 2 == 0);
    }
    // The always-dark module next to the lower-left finder.
    assert!(code.is_dark(8, size - 8));
    assert!(!code.is_dark(size, 0));
}

#[test]
fn svg_is_deterministic_and_accessible() {
    let code = QrCode::encode("https://example.com/?a=1&b=2", ErrorCorrection::Medium).unwrap();
    let svg = code.to_svg("Link to example.com & more", 4);
    assert_eq!(svg, code.to_svg("Link to example.com & more", 4));
    assert!(svg.starts_with("<svg xmlns=\"http://www.w3.org/2000/svg\""));
    assert!(svg.contains("role=\"img\""));
    assert!(svg.contains("<title>Link to example.com &amp; more</title>"));
    let side = code.size() + 8;
    assert!(svg.contains(&format!("viewBox=\"0 0 {side} {side}\"")));
}
