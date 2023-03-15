use mdbook::{
    book::{Book, BookItem, Chapter},
    errors::{Error, Result},
    preprocess::{Preprocessor, PreprocessorContext},
};
use serde::Deserialize;
use std::{
    io,
    path::{Path, PathBuf},
};
use walkdir::WalkDir;

type Date = chrono::naive::NaiveDate;

#[derive(Debug, Deserialize)]
struct PostsDirectory(String);

impl Default for PostsDirectory {
    fn default() -> Self {
        Self("posts".to_string())
    }
}

#[derive(Debug, Deserialize)]
struct ChapterName(String);

impl Default for ChapterName {
    fn default() -> Self {
        Self("Posts".to_string())
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum SortBy {
    Newest,
    Oldest,
    NameAZ,
    NameZA,
}

impl Default for SortBy {
    fn default() -> Self {
        Self::Newest
    }
}

#[derive(Debug, Default, Deserialize)]
#[serde(default, rename_all = "kebab-case")]
struct Config {
    directory: PostsDirectory,
    future: bool,
    chapter_name: ChapterName,
    sort_by: SortBy,
}

#[derive(Debug)]
struct Post {
    path: PathBuf,
    date: Date,
    name: String,
    parent_name: String,
}

impl Post {
    #[inline]
    fn new(path: PathBuf, date: Date, name: String, parent_name: String) -> Self {
        return Self {
            path,
            date,
            name,
            parent_name,
        };
    }
}

impl TryFrom<PathBuf> for Post {
    type Error = Error;
    fn try_from(path: PathBuf) -> Result<Self> {
        let content = std::fs::read_to_string(&path)?;
        //eprintln!("content: {}", content);
        //let content = String::new();
        Ok(Chapter::new(
            "test",
            content,
            post.path,
            vec![post.parent_name],
        ))
    }
}

impl TryFrom<Post> for Chapter {
    type Error = io::Error;
    fn try_from(post: Post) -> io::Result<Self> {
        let content = std::fs::read_to_string(&post.path)?;
        //eprintln!("content: {}", content);
        //let content = String::new();
        Ok(Chapter::new(
            "test",
            content,
            post.path,
            vec![post.parent_name],
        ))
    }
}

pub struct BlogPreprocessor;

impl BlogPreprocessor {
    pub fn new() -> Self {
        Self {}
    }
}

fn extract_date_from_filename<P: AsRef<Path>>(path: P) -> Result<Date> {
    let basename = path.as_ref().file_name().unwrap().to_string_lossy();

    log::info!("Extracting date from {}", basename);

    let mut index = 0;
    let mut count = 0;

    for (i, c) in basename.chars().enumerate() {
        if c == '-' {
            count += 1;
        }
        if count == 3 {
            index = i;
            break;
        }
    }

    log::info!("Date as string {}", &basename[..index]);

    let date = basename[..index].parse()?;

    Ok(date)
}

/// Returns a vector of [`Post`],
/// from a [`walkdir::WalkDir`].
///
/// A valid post ends is:
/// - a file;
/// - its name ends with '.md';
/// - and starts is formatted like YYYY-MM-DD-my-super-post.
fn collect_posts(walkdir: WalkDir, parent_name: String) -> Vec<Post> {
    walkdir
        .into_iter()
        .filter_map(|result| {
            match result {
                Ok(dir_entry) => {
                    let path_buf = dir_entry.into_path();
                    if path_buf.is_file() && path_buf.extension().map_or(false, |ext| ext == "md") {
                        match extract_date_from_filename(&path_buf) {
                            Ok(date) => Some(Post::new(path_buf, date, parent_name.clone())),
                            Err(e) => {
                                log::error!(
                                    "An error occured while extracting date from {path_buf:?}: {e}",
                                );
                                None
                            },
                        }
                    } else {
                        None
                    }
                },
                Err(e) => {
                    log::warn!("Some error occured reading directory entry: {e}");
                    None
                },
            }
        })
        .collect()
}

fn get_config(ctx: &PreprocessorContext) -> Config {
    ctx.config
        .get("preprocessor.blog")
        .map(|value| {
            match value.clone().try_into() {
                Ok(config) => config,
                Err(e) => {
                    log::error!(
                        "The [preprocessor.blog] section in book.toml contain invalid keys: {e}"
                    );
                    Config::default()
                },
            }
        })
        .unwrap_or_default()
}

impl Preprocessor for BlogPreprocessor {
    fn name(&self) -> &str {
        "blog"
    }

    fn run(&self, ctx: &PreprocessorContext, mut book: Book) -> Result<Book> {
        let src_dir = ctx.root.join(&ctx.config.book.src);

        let config: Config = get_config(ctx);

        let posts_dir = src_dir.join(&config.directory.0);

        log::info!("{posts_dir:?}");
        //let mut sections = &book.sections;

        /*
        let mut posts_chapter = Chapter::new(config.chapter, "".

        let mut posts_chapter: &mut Chapter = book
            .sections
            .iter()
            .filter_map(|book_item| {
                match book_item {
                    BookItem::Chapter(chapter) => Some(chapter),
                    _ => None,
                }
            })
            .find(|chapter| chapter.name == config.chapter_name.0)
            .ok_or(Error::msg(format!(
                "Could not find chapter \"{}\": did you forget to include a draft chapter using \
                 '- [{}]()' syntax in SUMMARY.md?",
                config.chapter_name.0, config.chapter_name.0
            )))?;
        */

        let walkdir = WalkDir::new(posts_dir);
        let mut posts = collect_posts(walkdir, config.chapter_name.0.clone());

        match config.sort_by {
            SortBy::Newest => posts.sort_by(|a, b| a.date.cmp(&b.date)),
            SortBy::Newest => posts.sort_by(|a, b| b.date.cmp(&a.date)),
            SortBy::NameAZ => posts.sort_by(|a, b| a.name.cmp(&b.name)),
            SortBy::NameZA => posts.sort_by(|a, b| b.name.cmp(&a.name)),
        }

        log::info!("Collected {posts:?} posts");

        for post in posts.into_iter() {
            let mut chapter: Chapter = post.try_into()?;
            chapter.parent_names = vec![config.chapter_name.0.clone()];
            chapter.source_path = Some(
                chapter
                    .source_path
                    .unwrap()
                    .strip_prefix(&src_dir)
                    .expect("This cannot fail")
                    .into(),
            );
            chapter.path = chapter.source_path.clone();
            //posts_chapter.sub_items.push(chapter.clone().into());
            log::info!("chapter: {:?}", chapter);
            book.push_item(chapter);
        }

        for item in book.iter() {
            if let BookItem::Chapter(ref ch) = *item {
                log::info!("{:?}", ch);
            }
        }

        Ok(book)
    }

    fn supports_renderer(&self, renderer: &str) -> bool {
        renderer == "html"
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn blog_preprocessor_run() {
        let input_json = r##"[
            {
                "root": "/path/to/book",
                "config": {
                    "book": {
                        "authors": ["AUTHOR"],
                        "language": "en",
                        "multilingual": false,
                        "src": "src",
                        "title": "TITLE"
                    },
                    "preprocessor": {
                        "blog": {
                            "directory": "blogs"
                        }
                    }
                },
                "renderer": "html",
                "mdbook_version": "0.4.28"
            },
            {
                "sections": [
                    {
                        "Chapter": {
                            "name": "Chapter 1",
                            "content": "# Chapter 1\n",
                            "number": [1],
                            "sub_items": [],
                            "path": "chapter_1.md",
                            "source_path": "chapter_1.md",
                            "parent_names": []
                        }
                    }
                ],
                "__non_exhaustive": null
            }
        ]"##;
        let input_json = input_json.as_bytes();

        let (ctx, book) = mdbook::preprocess::CmdPreprocessor::parse_input(input_json).unwrap();
        let expected_book = book.clone();
        let result = BlogPreprocessor::new().run(&ctx, book);
        assert!(result.is_ok());

        // The nop-preprocessor should not have made any changes to the book content.
        let actual_book = result.unwrap();
        assert_eq!(actual_book, expected_book);
    }
}
