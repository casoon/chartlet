"""Builds a small Sphinx project and a small MkDocs site and finds the chart in their pages."""

import subprocess
import sys
import textwrap

import pytest

import chartlet

from conftest import EXAMPLES

SPEC = (EXAMPLES / "monthly-revenue.json").read_text(encoding="utf-8")


def indented(text: str, prefix: str = "   ") -> str:
    return textwrap.indent(text, prefix)


def sphinx_build(source, output, *arguments):
    return subprocess.run(
        [sys.executable, "-m", "sphinx", "-q", "-b", "html", *arguments, source, output],
        capture_output=True,
        text=True,
    )


def test_sphinx_renders_the_directive_in_rst_and_myst(tmp_path):
    pytest.importorskip("sphinx")
    pytest.importorskip("myst_parser")
    source = tmp_path / "source"
    source.mkdir()
    (source / "conf.py").write_text(
        textwrap.dedent(
            """\
            extensions = ["myst_parser", "chartlet.sphinx"]
            myst_fence_as_directive = ["chartlet"]
            chartlet_options = {"table": "visible"}
            """
        )
    )
    (source / "index.rst").write_text(
        "Charts\n======\n\n.. toctree::\n\n   myst\n\n"
        f".. chartlet::\n   :id: rst-option\n\n{indented(SPEC)}\n\n"
        f".. chartlet:: id=rst-argument\n\n{indented(SPEC)}\n"
    )
    (source / "myst.md").write_text(
        f"# MyST\n\n```chartlet id=myst-fence\n{SPEC}\n```\n\n"
        f"```{{chartlet}}\n:id: myst-directive\n\n{SPEC}\n```\n"
    )
    output = tmp_path / "html"
    result = sphinx_build(source, output, "-W")
    assert result.returncode == 0, result.stderr
    index = (output / "index.html").read_text(encoding="utf-8")
    myst = (output / "myst.html").read_text(encoding="utf-8")
    for page, prefix in [
        (index, "rst-option"),
        (index, "rst-argument"),
        (myst, "myst-fence"),
        (myst, "myst-directive"),
    ]:
        assert chartlet.render(SPEC, format="html", table="visible", id_prefix=prefix) in page


def test_sphinx_reports_an_invalid_block_at_its_line(tmp_path):
    pytest.importorskip("sphinx")
    source = tmp_path / "source"
    source.mkdir()
    (source / "conf.py").write_text('extensions = ["chartlet.sphinx"]\n')
    (source / "index.rst").write_text("Charts\n======\n\n.. chartlet:: size=large\n\n   {}\n")
    result = sphinx_build(source, tmp_path / "html", "-W")
    assert result.returncode != 0
    assert "index.rst:4" in result.stderr
    assert 'chartlet: unknown fence setting "size"' in result.stderr


def test_mkdocs_renders_a_fenced_block(tmp_path):
    pytest.importorskip("mkdocs")
    (tmp_path / "docs").mkdir()
    (tmp_path / "mkdocs.yml").write_text(
        "site_name: Charts\nmarkdown_extensions:\n  - chartlet:\n      table: visible\n"
    )
    (tmp_path / "docs" / "index.md").write_text(
        f"# Charts\n\n```chartlet id=revenue\n{SPEC}\n```\n"
    )
    result = subprocess.run(
        [sys.executable, "-m", "mkdocs", "build", "--strict", "-q"],
        cwd=tmp_path,
        capture_output=True,
        text=True,
    )
    assert result.returncode == 0, result.stderr
    page = (tmp_path / "site" / "index.html").read_text(encoding="utf-8")
    assert chartlet.render(SPEC, format="html", table="visible", id_prefix="revenue") in page
