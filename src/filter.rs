//! File filtering module.
//!
//! This module provides functionality to filter out files that should not be
//! counted, such as binary files, configuration files, and documentation.

use std::path::Path;

/// Extensions for media/binary files to exclude.
const MEDIA_EXTENSIONS: &[&str] = &[
    "mp3", "png", "jpg", "jpeg", "gif", "svg", "woff2", "ico", "webp", "bmp", "tiff", "wav", "mp4",
    "avi", "mov", "webm", "flac", "ogg", "ttf", "woff", "eot", "otf", "pdf",
];

/// Extensions for data/configuration files to exclude.
const DATA_EXTENSIONS: &[&str] = &[
    "mmdb", "csv", "json", "toml", "lock", "ini", "yaml", "yml", "xml",
];

/// Extensions for documentation files to exclude.
const DOC_EXTENSIONS: &[&str] = &["md"];

/// Specific filenames to exclude.
const EXCLUDED_FILENAMES: &[&str] = &["LICENSE", "LICENSE-MIT", "LICENSE-APACHE", ".gitignore"];

/// A file filter that determines which files should be counted.
#[derive(Debug, Clone, Default)]
pub struct FileFilter {
    /// Additional extensions to exclude.
    extra_excluded_extensions: Vec<String>,
    /// Additional filenames to exclude.
    extra_excluded_filenames: Vec<String>,
}

impl FileFilter {
    /// Creates a new file filter with default exclusions.
    ///
    /// # Example
    ///
    /// ```
    /// use gitls::filter::FileFilter;
    ///
    /// let filter = FileFilter::new();
    /// assert!(filter.should_count("src/main.rs"));
    /// assert!(!filter.should_count("image.png"));
    /// ```
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds an extension to the exclusion list.
    ///
    /// # Arguments
    ///
    /// * `extension` - The extension to exclude (without the leading dot).
    ///
    /// # Example
    ///
    /// ```
    /// use gitls::filter::FileFilter;
    ///
    /// let filter = FileFilter::new().exclude_extension("log");
    /// assert!(!filter.should_count("debug.log"));
    /// ```
    pub fn exclude_extension(mut self, extension: impl Into<String>) -> Self {
        self.extra_excluded_extensions.push(extension.into());
        self
    }

    /// Adds a filename to the exclusion list.
    ///
    /// # Arguments
    ///
    /// * `filename` - The filename to exclude.
    ///
    /// # Example
    ///
    /// ```
    /// use gitls::filter::FileFilter;
    ///
    /// let filter = FileFilter::new().exclude_filename("Makefile.bak");
    /// assert!(!filter.should_count("Makefile.bak"));
    /// ```
    pub fn exclude_filename(mut self, filename: impl Into<String>) -> Self {
        self.extra_excluded_filenames.push(filename.into());
        self
    }

    /// Determines if a file should be counted based on its path.
    ///
    /// # Arguments
    ///
    /// * `path` - The path to the file.
    ///
    /// # Returns
    ///
    /// `true` if the file should be counted, `false` otherwise.
    ///
    /// # Example
    ///
    /// ```
    /// use gitls::filter::FileFilter;
    ///
    /// let filter = FileFilter::new();
    ///
    /// // Source code files should be counted
    /// assert!(filter.should_count("src/main.rs"));
    /// assert!(filter.should_count("lib/utils.py"));
    ///
    /// // Binary and config files should not be counted
    /// assert!(!filter.should_count("logo.png"));
    /// assert!(!filter.should_count("config.json"));
    /// assert!(!filter.should_count("README.md"));
    /// ```
    pub fn should_count(&self, path: impl AsRef<Path>) -> bool {
        let path = path.as_ref();

        // Check filename
        if let Some(filename) = path.file_name().and_then(|n| n.to_str()) {
            // Check against default excluded filenames
            if EXCLUDED_FILENAMES.contains(&filename) {
                return false;
            }

            // Check against extra excluded filenames
            if self.extra_excluded_filenames.iter().any(|e| e == filename) {
                return false;
            }
        }

        // Check extension
        if let Some(extension) = path.extension().and_then(|e| e.to_str()) {
            let ext_lower = extension.to_lowercase();

            // Check against default excluded extensions
            if MEDIA_EXTENSIONS.iter().any(|&e| e == ext_lower)
                || DATA_EXTENSIONS.iter().any(|&e| e == ext_lower)
                || DOC_EXTENSIONS.iter().any(|&e| e == ext_lower)
            {
                return false;
            }

            // Check against extra excluded extensions
            if self
                .extra_excluded_extensions
                .iter()
                .any(|e| e.to_lowercase() == ext_lower)
            {
                return false;
            }
        }

        true
    }

    /// Filters a list of file paths, returning only those that should be counted.
    ///
    /// # Arguments
    ///
    /// * `files` - An iterator of file paths.
    ///
    /// # Returns
    ///
    /// A vector of file paths that should be counted.
    ///
    /// # Example
    ///
    /// ```
    /// use gitls::filter::FileFilter;
    ///
    /// let filter = FileFilter::new();
    /// let files = vec![
    ///     "src/main.rs".to_string(),
    ///     "logo.png".to_string(),
    ///     "README.md".to_string(),
    /// ];
    ///
    /// let filtered = filter.filter_files(files);
    /// assert_eq!(filtered, vec!["src/main.rs".to_string()]);
    /// ```
    pub fn filter_files<I, S>(&self, files: I) -> Vec<String>
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        files
            .into_iter()
            .filter(|f| self.should_count(f.as_ref()))
            .map(|f| f.as_ref().to_string())
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_should_count_source_files() {
        let filter = FileFilter::new();

        assert!(filter.should_count("main.rs"));
        assert!(filter.should_count("lib.py"));
        assert!(filter.should_count("index.js"));
        assert!(filter.should_count("app.tsx"));
        assert!(filter.should_count("Makefile"));
        assert!(filter.should_count("src/utils/helper.go"));
    }

    #[test]
    fn test_should_not_count_media_files() {
        let filter = FileFilter::new();

        assert!(!filter.should_count("image.png"));
        assert!(!filter.should_count("photo.jpg"));
        assert!(!filter.should_count("photo.JPEG")); // Case insensitive
        assert!(!filter.should_count("icon.svg"));
        assert!(!filter.should_count("audio.mp3"));
        assert!(!filter.should_count("font.woff2"));
        assert!(!filter.should_count("favicon.ico"));
    }

    #[test]
    fn test_should_not_count_data_files() {
        let filter = FileFilter::new();

        assert!(!filter.should_count("data.csv"));
        assert!(!filter.should_count("config.json"));
        assert!(!filter.should_count("settings.toml"));
        assert!(!filter.should_count("Cargo.lock"));
        assert!(!filter.should_count("database.mmdb"));
    }

    #[test]
    fn test_should_not_count_doc_files() {
        let filter = FileFilter::new();

        assert!(!filter.should_count("README.md"));
        assert!(!filter.should_count("CHANGELOG.md"));
        assert!(!filter.should_count("docs/guide.md"));
    }

    #[test]
    fn test_should_not_count_excluded_filenames() {
        let filter = FileFilter::new();

        assert!(!filter.should_count("LICENSE"));
        assert!(!filter.should_count(".gitignore"));
        assert!(!filter.should_count("path/to/LICENSE"));
    }

    #[test]
    fn test_custom_exclusions() {
        let filter = FileFilter::new()
            .exclude_extension("log")
            .exclude_filename("custom.txt");

        assert!(!filter.should_count("debug.log"));
        assert!(!filter.should_count("custom.txt"));
        assert!(filter.should_count("main.rs"));
    }

    #[test]
    fn test_filter_files() {
        let filter = FileFilter::new();
        let files = vec![
            "src/main.rs",
            "src/lib.rs",
            "image.png",
            "README.md",
            "config.json",
            "LICENSE",
        ];

        let filtered = filter.filter_files(files);

        assert_eq!(filtered.len(), 2);
        assert!(filtered.contains(&"src/main.rs".to_string()));
        assert!(filtered.contains(&"src/lib.rs".to_string()));
    }

    #[test]
    fn test_case_insensitive_extension() {
        let filter = FileFilter::new();

        assert!(!filter.should_count("image.PNG"));
        assert!(!filter.should_count("image.Png"));
        assert!(!filter.should_count("data.JSON"));
    }
}
