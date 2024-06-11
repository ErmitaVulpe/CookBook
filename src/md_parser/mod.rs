pub use pulldown_cmark::Options;
use pulldown_cmark::{
    BlockQuoteKind,
    CodeBlockKind,
    Event,
    HeadingLevel,
    Parser,
    LinkType,
    MetadataBlockKind,
    Tag,
    TagEnd,
    TextMergeStream
};
use leptos::{view, IntoView, View};
use std::ops::Deref;

pub fn deafult_options() -> Options {
    Options::ENABLE_STRIKETHROUGH |
    Options::ENABLE_TASKLISTS |
    Options::ENABLE_SMART_PUNCTUATION |
    Options::ENABLE_HEADING_ATTRIBUTES |
    Options::ENABLE_MATH
}

pub fn parse(text: &str, options: Options) -> impl IntoView {
    let parser = Parser::new_ext(text, options);
    let mut iter = TextMergeStream::new(parser);
    parse_recursively(&mut iter, None)
}

fn parse_recursively<'a, I>(iter: &mut I, scope: Option<TagEnd>) -> Vec<View>
where I: Iterator<Item = Event<'a>> {
    let mut elements = Vec::new();

    while let Some(item) = iter.next() {
        match item {
            Event::Start(tag) => {
                let children = parse_recursively(iter, Some(tag.clone().into()));

                elements.push(match tag {
                    Tag::Paragraph => view! {<p>{children}</p>}.into_view(),
                    Tag::Heading { level, id, classes, attrs: _ } => {
                        // TODO Find a better way to do this
                        let id = id.as_ref().map(|s| s.to_string());
                        let classes = classes
                            .iter()
                            .map(|s| s.deref())
                            .collect::<Vec<_>>()
                            .join(" ");

                        match level { 
                            HeadingLevel::H1 => view! {<h1 id=id class=classes>{children}</h1>}.into_view(),
                            HeadingLevel::H2 => view! {<h2 id=id class=classes>{children}</h2>}.into_view(),
                            HeadingLevel::H3 => view! {<h3 id=id class=classes>{children}</h3>}.into_view(),
                            HeadingLevel::H4 => view! {<h4 id=id class=classes>{children}</h4>}.into_view(),
                            HeadingLevel::H5 => view! {<h5 id=id class=classes>{children}</h5>}.into_view(),
                            HeadingLevel::H6 => view! {<h6 id=id class=classes>{children}</h6>}.into_view(),
                        }
                    },
                    Tag::BlockQuote(kind) => {
                        let class = kind.map(|k| match k {
                            BlockQuoteKind::Note => "markdown-alert-Note",
                            BlockQuoteKind::Tip => "markdown-alert-Tip",
                            BlockQuoteKind::Important => "markdown-alert-Important",
                            BlockQuoteKind::Warning => "markdown-alert-Warning",
                            BlockQuoteKind::Caution => "markdown-alert-Caution",
                        });
                        view! {<blockquote class=class>{children}</blockquote>}.into_view()
                    },
                    Tag::CodeBlock(kind) => match kind {
                        CodeBlockKind::Indented => view! {<pre><code>{children}</code></pre>}.into_view(),
                        CodeBlockKind::Fenced(lang) => view! {<pre><code class=format!("language-{lang}")>{children}</code></pre>}.into_view(),
                    },
                    Tag::HtmlBlock => ().into_view(),
                    Tag::List(start) => match start {
                        None => view! { <ul>{children}</ul> }.into_view(),
                        Some(1) => view! { <ol>{children}</ol> }.into_view(),
                        Some(start) => view! { <ol start=start>{children}</ol> }.into_view(),
                    },
                    Tag::Item => view! {<li>{children}</li>}.into_view(),
                    Tag::FootnoteDefinition(_) => todo!(),
                    Tag::Table(_) => todo!(), // TODO support table
                    Tag::TableHead => todo!(),
                    Tag::TableRow => todo!(),
                    Tag::TableCell => todo!(),
                    Tag::Emphasis => view! {<em>{children}</em>}.into_view(),
                    Tag::Strong => view! {<strong>{children}</strong>}.into_view(),
                    Tag::Strikethrough => view! {<del>{children}</del>}.into_view(),
                    Tag::Link { link_type, dest_url, title, id } => match link_type {
                        LinkType::Email => view! {
                            <a
                                href=format!("mailto:{dest_url}")
                                id=id.to_string()
                                title=title.to_string()
                            >{children}</a>
                        }.into_view(),
                        _ => view! {
                            <a
                                href=dest_url.to_string()
                                id=id.to_string()
                                title=title.to_string()
                                target="_blank"
                            >{children}</a>
                        }.into_view(),
                    },
                    Tag::Image { link_type: _, dest_url, title, id } => view! {
                        <img
                            src=dest_url.to_string()
                            title=if title.is_empty() {
                                None
                            } else {
                                Some(title.to_string())
                            }
                            id=if id.is_empty() {
                                None
                            } else {
                                Some(id.to_string())
                            }
                            alt=match &children.first() {
                                Some(View::Text(val)) => Some(val.content.to_string()),
                                _ => None,
                            }
                        />
                    }.into_view(),
                    Tag::MetadataBlock(kind) => match kind {
                        MetadataBlockKind::YamlStyle => view! {
                            <div>{children}</div>
                        }.into_view(),
                        MetadataBlockKind::PlusesStyle => todo!(),
                    },
                });
            },
            Event::End(tag) => if Some(tag) == scope {
                return elements;
            },
            Event::Text(val) => elements.push(val.to_string().into_view()),
            Event::Code(val) => elements.push(view! {<code>{val.to_string()}</code>}.into_view()),
            Event::InlineMath(val) => elements.push(view! {<span class="math math-inline">{val.to_string()}</span>}.into_view()),
            Event::DisplayMath(val) => elements.push(view! {<span class="math math-display">{val.to_string()}</span>}.into_view()),
            Event::Html(val) | Event::InlineHtml(val) => elements.push(val.to_string().into_view()), // TODO support inline html
            Event::FootnoteReference(_) => todo!(),
            Event::SoftBreak => elements.push(" ".into_view()),
            Event::HardBreak => elements.push(view! {<br/>}.into_view()),
            Event::Rule => elements.push(view! {<hr/>}.into_view()),
            Event::TaskListMarker(val) => elements.push(view! {<input disabled=true type="checkbox" checked=val />}.into_view()),
        }
    }

    elements
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{
        str::from_utf8,
        fs,
    };
    use leptos::ssr::render_to_string;

    #[test]
    fn parse_spec() {
        let spec_bytes = fs::read("src/md_parser/spec.md").unwrap();
        let spec = from_utf8(&spec_bytes).unwrap().to_string();

        let options = Options::all();

        println!("{}", render_to_string(move || parse(&spec, options)));
    }
}
