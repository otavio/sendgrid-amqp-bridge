// Copyright (C) 2018 O.S. Systems Sofware LTDA
//
// SPDX-License-Identifier: MIT

use git_version::git_version;

/// Returns the version in use, including the commit and if there is
/// uncommited modification in the source.
///
/// Internally, it use `git describe` to get the version and the
/// number of changes since the last tag.
pub const fn version() -> &'static str {
    git_version!()
}
