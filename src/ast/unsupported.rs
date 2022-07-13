pub(crate) const UNSUPPORTED_NOTATIONS: [&str; 4] = [
    "class", // The parser is not file-aware, only comment-aware
    "def", // Bindgen doesn't support constant defs anyway
    "enum", // The parser is not file-aware, only comment-aware
    "vhdlflow", // This can't be implemented
];