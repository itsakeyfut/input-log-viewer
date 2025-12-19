//! Application error types for user-facing error handling.
//!
//! This module defines error types that are designed to be displayed to users
//! in error dialogs, with detailed information and recovery options.

// Allow dead code for error variants that are designed for future use
#![allow(dead_code)]

use std::path::PathBuf;
use thiserror::Error;

/// Application-level errors that can be displayed to users.
///
/// These errors are designed to provide clear, actionable information
/// to help users understand and resolve issues.
#[derive(Debug, Clone, Error)]
pub enum AppError {
    /// File was not found at the specified path
    #[error("File not found")]
    FileNotFound {
        /// Path to the file that was not found
        path: PathBuf,
    },

    /// File exists but cannot be read (permissions, locked, etc.)
    #[error("Cannot read file")]
    FileReadError {
        /// Path to the file that could not be read
        path: PathBuf,
        /// Reason for the failure
        reason: String,
    },

    /// File format is invalid or corrupted
    #[error("Invalid file format")]
    InvalidFormat {
        /// Path to the file with invalid format
        path: Option<PathBuf>,
        /// Description of what's wrong
        message: String,
        /// Line number where the error occurred (1-indexed, for JSON)
        line: Option<usize>,
        /// Column/position where the error occurred
        column: Option<usize>,
    },

    /// File version is not supported
    #[error("Unsupported version")]
    UnsupportedVersion {
        /// Path to the file with unsupported version
        path: Option<PathBuf>,
        /// Version found in the file
        found: u32,
        /// Version supported by this application
        supported: u32,
    },

    /// File extension is not recognized
    #[error("Unsupported file type")]
    UnsupportedFileType {
        /// Path to the file
        path: PathBuf,
        /// Expected file extensions
        expected: Vec<String>,
    },

    /// Generic I/O error
    #[error("I/O error")]
    IoError {
        /// Path related to the error, if any
        path: Option<PathBuf>,
        /// Description of what went wrong
        reason: String,
    },

    /// Settings could not be saved
    #[error("Settings save error")]
    SettingsSaveError {
        /// Description of the failure
        reason: String,
    },

    /// Settings could not be loaded
    #[error("Settings load error")]
    SettingsLoadError {
        /// Description of the failure
        reason: String,
    },
}

impl AppError {
    /// Returns true if the error is recoverable (user can continue using the app).
    ///
    /// Recoverable errors allow the user to dismiss the dialog and try again
    /// or continue using other features. Non-recoverable errors typically
    /// require the application to restart.
    pub fn is_recoverable(&self) -> bool {
        matches!(
            self,
            Self::FileNotFound { .. }
                | Self::FileReadError { .. }
                | Self::InvalidFormat { .. }
                | Self::UnsupportedVersion { .. }
                | Self::UnsupportedFileType { .. }
                | Self::IoError { .. }
                | Self::SettingsSaveError { .. }
                | Self::SettingsLoadError { .. }
        )
    }

    /// Returns true if this error supports retry operation.
    ///
    /// File-related errors can potentially be retried after the user
    /// fixes the underlying issue (e.g., file permissions, file location).
    pub fn supports_retry(&self) -> bool {
        matches!(
            self,
            Self::FileNotFound { .. }
                | Self::FileReadError { .. }
                | Self::IoError { path: Some(_), .. }
        )
    }

    /// Get the file path associated with this error, if any.
    pub fn file_path(&self) -> Option<&PathBuf> {
        match self {
            Self::FileNotFound { path } => Some(path),
            Self::FileReadError { path, .. } => Some(path),
            Self::InvalidFormat { path, .. } => path.as_ref(),
            Self::UnsupportedVersion { path, .. } => path.as_ref(),
            Self::UnsupportedFileType { path, .. } => Some(path),
            Self::IoError { path, .. } => path.as_ref(),
            Self::SettingsSaveError { .. } | Self::SettingsLoadError { .. } => None,
        }
    }

    /// Get the title for the error dialog.
    pub fn dialog_title(&self) -> &'static str {
        match self {
            Self::FileNotFound { .. } => "File Not Found",
            Self::FileReadError { .. } => "Cannot Read File",
            Self::InvalidFormat { .. } => "Invalid File Format",
            Self::UnsupportedVersion { .. } => "Unsupported Version",
            Self::UnsupportedFileType { .. } => "Unsupported File Type",
            Self::IoError { .. } => "I/O Error",
            Self::SettingsSaveError { .. } => "Settings Error",
            Self::SettingsLoadError { .. } => "Settings Error",
        }
    }

    /// Get a brief description of the error suitable for display.
    pub fn brief_description(&self) -> String {
        match self {
            Self::FileNotFound { path } => {
                format!(
                    "The file '{}' could not be found.",
                    path.file_name()
                        .map(|n| n.to_string_lossy().to_string())
                        .unwrap_or_else(|| path.to_string_lossy().to_string())
                )
            }
            Self::FileReadError { path, .. } => {
                format!(
                    "Could not read the file '{}'.",
                    path.file_name()
                        .map(|n| n.to_string_lossy().to_string())
                        .unwrap_or_else(|| path.to_string_lossy().to_string())
                )
            }
            Self::InvalidFormat { message, .. } => message.clone(),
            Self::UnsupportedVersion {
                found, supported, ..
            } => {
                format!(
                    "File version {} is not supported. This viewer supports version {}.",
                    found, supported
                )
            }
            Self::UnsupportedFileType { expected, .. } => {
                format!(
                    "Please use a file with one of these extensions: {}",
                    expected.join(", ")
                )
            }
            Self::IoError { reason, .. } => reason.clone(),
            Self::SettingsSaveError { reason } => format!("Could not save settings: {}", reason),
            Self::SettingsLoadError { reason } => format!("Could not load settings: {}", reason),
        }
    }

    /// Get detailed error information for technical support / bug reports.
    ///
    /// This includes full paths, line numbers, and other technical details
    /// that can help diagnose issues.
    pub fn detailed_info(&self) -> String {
        let mut lines = Vec::new();

        lines.push(format!("Error Type: {}", self.dialog_title()));
        lines.push(format!("Description: {}", self.brief_description()));

        match self {
            Self::FileNotFound { path } => {
                lines.push(format!("Full Path: {}", path.display()));
            }
            Self::FileReadError { path, reason } => {
                lines.push(format!("Full Path: {}", path.display()));
                lines.push(format!("Reason: {}", reason));
            }
            Self::InvalidFormat {
                path,
                message,
                line,
                column,
            } => {
                if let Some(p) = path {
                    lines.push(format!("File: {}", p.display()));
                }
                if let Some(l) = line {
                    lines.push(format!("Line: {}", l));
                }
                if let Some(c) = column {
                    lines.push(format!("Column: {}", c));
                }
                lines.push(format!("Details: {}", message));
            }
            Self::UnsupportedVersion {
                path,
                found,
                supported,
            } => {
                if let Some(p) = path {
                    lines.push(format!("File: {}", p.display()));
                }
                lines.push(format!("Found Version: {}", found));
                lines.push(format!("Supported Version: {}", supported));
            }
            Self::UnsupportedFileType { path, expected } => {
                lines.push(format!("File: {}", path.display()));
                lines.push(format!("Supported Extensions: {}", expected.join(", ")));
            }
            Self::IoError { path, reason } => {
                if let Some(p) = path {
                    lines.push(format!("Path: {}", p.display()));
                }
                lines.push(format!("Details: {}", reason));
            }
            Self::SettingsSaveError { reason } | Self::SettingsLoadError { reason } => {
                lines.push(format!("Details: {}", reason));
            }
        }

        lines.join("\n")
    }
}

/// Create an AppError from a file path and I/O error.
pub fn from_io_error(path: PathBuf, error: std::io::Error) -> AppError {
    match error.kind() {
        std::io::ErrorKind::NotFound => AppError::FileNotFound { path },
        std::io::ErrorKind::PermissionDenied => AppError::FileReadError {
            path,
            reason: "Permission denied".to_string(),
        },
        _ => AppError::FileReadError {
            path,
            reason: error.to_string(),
        },
    }
}

/// Create an AppError from a parse error with optional path context.
pub fn from_parse_error(
    path: Option<PathBuf>,
    error: &crate::core::parser::ParseError,
) -> AppError {
    use crate::core::parser::ParseError;

    match error {
        ParseError::JsonSyntax(e) => {
            // Try to extract line/column from serde_json error
            let (line, column) = extract_json_position(e);
            AppError::InvalidFormat {
                path,
                message: e.to_string(),
                line,
                column,
            }
        }
        ParseError::MissingField { field } => AppError::InvalidFormat {
            path,
            message: format!("Missing required field: {}", field),
            line: None,
            column: None,
        },
        ParseError::InvalidColor { value } => AppError::InvalidFormat {
            path,
            message: format!(
                "Invalid color format '{}': expected hex color like #RRGGBB",
                value
            ),
            line: None,
            column: None,
        },
        ParseError::InvalidEnumValue {
            field,
            value,
            expected,
        } => AppError::InvalidFormat {
            path,
            message: format!(
                "Invalid {} value '{}': expected one of {}",
                field, value, expected
            ),
            line: None,
            column: None,
        },
        ParseError::UnsupportedVersion { version } => AppError::UnsupportedVersion {
            path,
            found: *version,
            supported: 1, // Current supported version
        },
        ParseError::InvalidMagic { found } => AppError::InvalidFormat {
            path,
            message: format!("Invalid file signature: expected 'ILOG', found '{}'", found),
            line: None,
            column: None,
        },
        ParseError::FileTooSmall { expected, found } => AppError::InvalidFormat {
            path,
            message: format!(
                "File is too small: expected at least {} bytes, found {}",
                expected, found
            ),
            line: None,
            column: None,
        },
        ParseError::InvalidBinaryEvent { index, reason } => AppError::InvalidFormat {
            path,
            message: format!("Invalid event at index {}: {}", index, reason),
            line: None,
            column: None,
        },
        ParseError::EventCountMismatch {
            header_count,
            actual_count,
        } => AppError::InvalidFormat {
            path,
            message: format!(
                "Event count mismatch: header declares {} events, but file contains {}",
                header_count, actual_count
            ),
            line: None,
            column: None,
        },
    }
}

/// Extract line and column from a serde_json::Error if available.
fn extract_json_position(error: &serde_json::Error) -> (Option<usize>, Option<usize>) {
    // serde_json::Error has line() and column() methods
    let line = error.line();
    let column = error.column();

    // line() returns 0 if not applicable, otherwise 1-indexed
    let line = if line > 0 { Some(line) } else { None };
    let column = if column > 0 { Some(column) } else { None };

    (line, column)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_file_not_found_error() {
        let error = AppError::FileNotFound {
            path: PathBuf::from("/path/to/file.ilj"),
        };
        assert!(error.is_recoverable());
        assert!(error.supports_retry());
        assert_eq!(error.dialog_title(), "File Not Found");
        assert!(error.file_path().is_some());
    }

    #[test]
    fn test_invalid_format_error() {
        let error = AppError::InvalidFormat {
            path: Some(PathBuf::from("/path/to/file.ilj")),
            message: "Invalid JSON".to_string(),
            line: Some(10),
            column: Some(5),
        };
        assert!(error.is_recoverable());
        assert!(!error.supports_retry());
        assert_eq!(error.dialog_title(), "Invalid File Format");
        let details = error.detailed_info();
        assert!(details.contains("Line: 10"));
        assert!(details.contains("Column: 5"));
    }

    #[test]
    fn test_unsupported_version_error() {
        let error = AppError::UnsupportedVersion {
            path: None,
            found: 99,
            supported: 1,
        };
        assert!(error.is_recoverable());
        assert!(!error.supports_retry());
        assert_eq!(error.dialog_title(), "Unsupported Version");
        assert!(error.brief_description().contains("99"));
    }

    #[test]
    fn test_from_io_error_not_found() {
        let path = PathBuf::from("/test/file.ilj");
        let io_error = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
        let error = from_io_error(path.clone(), io_error);

        match error {
            AppError::FileNotFound { path: p } => assert_eq!(p, path),
            _ => panic!("Expected FileNotFound error"),
        }
    }

    #[test]
    fn test_from_io_error_permission_denied() {
        let path = PathBuf::from("/test/file.ilj");
        let io_error = std::io::Error::new(std::io::ErrorKind::PermissionDenied, "access denied");
        let error = from_io_error(path.clone(), io_error);

        match error {
            AppError::FileReadError { path: p, reason } => {
                assert_eq!(p, path);
                assert!(reason.contains("Permission"));
            }
            _ => panic!("Expected FileReadError error"),
        }
    }
}
