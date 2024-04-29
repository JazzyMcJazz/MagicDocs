use comrak::{
    markdown_to_html_with_plugins, plugins::syntect::SyntectAdapter, ComrakOptions, ComrakPlugins,
};

const CODE_BLOCK_THEME: &str = "base16-eighties.dark";

pub struct Markdown;

impl Markdown {
    pub fn to_html(&self, markdown: &str) -> String {
        let adapter = SyntectAdapter::new(Some(CODE_BLOCK_THEME));
        let options = ComrakOptions::default();
        let mut plugins = ComrakPlugins::default();
        plugins.render.codefence_syntax_highlighter = Some(&adapter);

        markdown_to_html_with_plugins(markdown, &options, &plugins)
    }
}
