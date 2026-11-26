import logging

import pytest

markdown = pytest.importorskip("markdown")

import chartlet  # noqa: E402

from conftest import EXAMPLES  # noqa: E402

SPEC = (EXAMPLES / "monthly-revenue.json").read_text(encoding="utf-8")


def convert(text: str, **config) -> str:
    return markdown.markdown(text, extensions=["chartlet"], extension_configs={"chartlet": config})


def figure(**options) -> str:
    return chartlet.render(SPEC, format="html", **options)


def test_a_block_becomes_the_html_figure_of_the_cli():
    html = convert(f"Before.\n\n```chartlet id=revenue\n{SPEC}\n```\n\nAfter.\n")
    assert html == f"<p>Before.</p>\n{figure(id_prefix='revenue')}\n<p>After.</p>"


def test_options_apply_to_every_block_and_settings_to_their_own():
    html = convert(
        f"```chartlet\n{SPEC}\n```\n\n~~~~chartlet id='second'\n{SPEC}\n~~~~\n",
        table="visible",
        theme="dark",
    )
    assert figure(table="visible", theme="dark") in html
    assert figure(table="visible", theme="dark", id_prefix="second") in html


def test_other_fences_and_blocks_inside_them_stay_code():
    text = f"````md\n```chartlet\n{SPEC}\n```\n````\n\n```json\n{{}}\n```\n"
    html = convert(text)
    assert "<figure" not in html
    assert html == markdown.markdown(text)


def test_a_block_inside_a_list_item():
    indented = "\n".join(f"    {line}" for line in f"```chartlet\n{SPEC}\n```".splitlines())
    html = convert(f"- First\n\n{indented}\n")
    assert figure() in html
    assert html.startswith("<ul>\n<li>\n<p>First</p>\n<figure")


def test_another_language_name():
    html = convert(f"```chart\n{SPEC}\n```\n", lang="chart")
    assert html == figure()


def test_an_invalid_block_names_its_number():
    with pytest.raises(ValueError, match=r"chartlet block 2: invalid_json"):
        convert(f"```chartlet\n{SPEC}\n```\n\n```chartlet\n{{\n```\n")
    with pytest.raises(ValueError, match="chartlet block 1: unknown fence setting"):
        convert(f"```chartlet size=large\n{SPEC}\n```\n")


def test_warnings_go_to_the_log(caplog):
    spec = '{"schemaVersion":1,"type":"bar","title":"Dense","width":320,"data":[%s]}' % ",".join(
        f'{{"label":"Category number {index}","value":{index}}}' for index in range(40)
    )
    with caplog.at_level(logging.WARNING, logger="mkdocs.extensions.chartlet"):
        convert(f"```chartlet\n{spec}\n```\n")
    assert caplog.records
    assert caplog.records[0].getMessage().startswith("chartlet block 1: warning[")
