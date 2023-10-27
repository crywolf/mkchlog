mod mocks;

use mkchlog::changelog;
use mkchlog::changelog::Changelog;
use mkchlog::git::Git;
use mkchlog::template::Template;
use mocks::GitCmdMock;
use std::fs::File;

const YAML_FILE: &str = "tests/mkchlog.yml";

#[test]
fn it_produces_correct_output() {
    let mocked_log = String::from(
        "\
commit b532ebcb0a214fbc69a5f5138e43eec14ea1a9dc
Author: Cry Wolf <cry.wolf@centrum.cz>
Date:   Tue Oct 24 19:17:09 2023 +0200

    Setup CI

    changelog:
        section: dev
        inherit: title
        title-is-enough: true

commit cdbfeb9b2576e07f12da569c54f5ec3cd7b9c0fc
Author: Cry Wolf <cry.wolf@centrum.cz>
Date:   Sun Oct 22 23:08:57 2023 +0200

    Allow configuring commit ID in yaml

    This adds a field `skip-commits-up-to` into top level of yaml config so
    that users don't have to remember what to supply in `-c` argument every
    time.

    changelog:
        section: features
        inherit: all

commit 22e27ce785698c4a873eb5e2ad9e0cf9c849be8d
Author: Martin Habovstiak <martin.habovstiak@gmail.com>
Date:   Sun Oct 22 09:12:50 2023 +0200

    Support building on Debian Bookworm

    This change lowers the requirements for dependencies so that the code
    compiles on Rust 1.63 which is in Debian Bookworm. Further, the
    dependencies are lowered such that the packages vendored in Debian
    Bookworm can be used directly.

    This uses version ranges so that the newest crates can still be used
    (they didn't break our code).

    changelog:
        section: features
        title-is-enough: true

commit 624c947820cba6c0665b84bfc139f209277f2a95
Author: Martin Habovstiak <martin.habovstiak@gmail.com>
Date:   Sat Oct 21 19:00:27 2023 +0200

    Setup Github Actions

    This configures github actions to test `mkchlog` as well as run it on
    its own repository. Also moved `.mkchlog.yml`, which was used in test,
    to `tests/mkchlog.yml` and created custom `.mkchlog.yml` that's used in
    this project.

    The new `.mkchlog.yml` is heavily inspired by the original example with
    more sections, so we're more flexible in the future. Includes a section
    used in this commit. :)

    changelog:
            section: dev
            title-is-enough: true

commit 1cc72956df91e2fd8c45e72983c4e1149f1ac3b3
Author: Cry Wolf <cry.wolf@centrum.cz>
Date:   Tue Jun 13 16:27:59 2023 +0200

    Fixed TOCTOU race condition when opening file

    Previously we checked the file permissions before opening
    the file now we check the metadata using file descriptor
    after opening the file. (before reading)

    changelog:
        section: security:vuln_fixes
        title: Fixed vulnerability related to opening files
        description: The application was vulnerable to attacks
                     if the attacker had access to the working
                     directory. If you run this in such
                     enviroment you should update ASAP. If your
                     working directory is **not** accessible by
                     unprivileged users you don't need to worry.

commit 7c85bee4303d56bededdfacf8fbb7bdc68e2195b
Author: Cry Wolf <cry.wolf@centrum.cz>
Date:   Tue Jun 13 16:26:35 2023 +0200

    Don't reallocate the buffer when we know its size

    This computes the size and allocates the buffer upfront.
    Avoiding allocations like this introduces 10% speedup.

    changelog:
        section: perf
        title: Improved processing speed by 10%
        title-is-enough: true

commit a1a654e256cc96e1c4b5a81845b5e3f3f0aa9ed3
Author: Cry Wolf <cry.wolf@centrum.cz>
Date:   Tue Jun 13 16:25:29 2023 +0200

    Fixed grammar mistakes.

    We found 42 grammar mistakes that are fixed in this commit.

    changelog: skip

commit 62db026b0ead7f0659df10c70e402c70ede5d7dd
Author: Cry Wolf <cry.wolf@centrum.cz>
Date:   Tue Jun 13 16:24:22 2023 +0200

    Added ability to skip commits.

    This allows commits to be skipped by typing 'changelog: skip'
    at the end of the commit. This is mainly useful for typo
    fixes or other things irrelevant to the user of a project.

    changelog:
        inherit: all
        section: features",
    );

    let git_cmd = Box::new(GitCmdMock::new(mocked_log));
    let git = Git::new(git_cmd);

    let f = File::open(YAML_FILE).unwrap();
    let template = Template::<changelog::Changes>::new(f).unwrap();
    let changelog = Changelog::new(template, git);

    let output = changelog.produce().unwrap();

    let exp_output = "\
============================================

## Security

This section contains very important security-related changes.

### Fixed vulnerabilities

#### Fixed vulnerability related to opening files

The application was vulnerable to attacks if the attacker had access to the working directory. \
If you run this in such enviroment you should update ASAP. \
If your working directory is **not** accessible by unprivileged users you don't need to worry.

## New features

* Support building on Debian Bookworm

### Allow configuring commit ID in yaml

This adds a field `skip-commits-up-to` into top level of yaml config so that users don't have to remember what to supply in `-c` argument every time.

### Added ability to skip commits.

This allows commits to be skipped by typing 'changelog: skip' \
at the end of the commit. This is mainly useful for typo fixes or other things irrelevant to the user of a project.

## Performance improvements

* Improved processing speed by 10%

## Development

Internal development changes

* Setup CI

* Setup Github Actions

============================================";

    assert_eq!(exp_output, output);
}

#[test]
fn only_sections_with_commits_shoud_be_printed_out() {
    let mocked_log = String::from(
        "\
commit 1cc72956df91e2fd8c45e72983c4e1149f1ac3b3
Author: Cry Wolf <cry.wolf@centrum.cz>
Date:   Tue Jun 13 16:27:59 2023 +0200

    Fixed TOCTOU race condition when opening file

    Previously we checked the file permissions before opening
    the file now we check the metadata using file descriptor
    after opening the file. (before reading)

    changelog:
        section: security:vuln_fixes
        title: Fixed vulnerability related to opening files
        description: The application was vulnerable to attacks
                     if the attacker had access to the working
                     directory. If you run this in such
                     enviroment you should update ASAP. If your
                     working directory is **not** accessible by
                     unprivileged users you don't need to worry.

",
    );

    let git_cmd = Box::new(GitCmdMock::new(mocked_log));
    let git = Git::new(git_cmd);

    let f = File::open(YAML_FILE).unwrap();
    let template = Template::<changelog::Changes>::new(f).unwrap();
    let changelog = Changelog::new(template, git);

    let output = changelog.produce().unwrap();

    let exp_output = "\
============================================

## Security

This section contains very important security-related changes.

### Fixed vulnerabilities

#### Fixed vulnerability related to opening files

The application was vulnerable to attacks if the attacker had access to the working directory. \
If you run this in such enviroment you should update ASAP. \
If your working directory is **not** accessible by unprivileged users you don't need to worry.

============================================";

    assert_eq!(exp_output, output);
}

#[test]
fn commits_with_title_only_shoud_be_printed_before_commits_with_description() {
    let mocked_log = String::from(
        "\
commit cdbfeb9b2576e07f12da569c54f5ec3cd7b9c0fc
Author: Cry Wolf <cry.wolf@centrum.cz>
Date:   Sun Oct 22 23:08:57 2023 +0200

    Allow configuring commit ID in yaml

    This adds a field `skip-commits-up-to` into top level of yaml config so
    that users don't have to remember what to supply in `-c` argument every
    time.

    changelog:
        section: features
        inherit: all

commit 22e27ce785698c4a873eb5e2ad9e0cf9c849be8d
Author: Martin Habovstiak <martin.habovstiak@gmail.com>
Date:   Sun Oct 22 09:12:50 2023 +0200

    Support building on Debian Bookworm

    This change lowers the requirements for dependencies so that the code
    compiles on Rust 1.63 which is in Debian Bookworm. Further, the
    dependencies are lowered such that the packages vendored in Debian
    Bookworm can be used directly.

    This uses version ranges so that the newest crates can still be used
    (they didn't break our code).

    changelog:
            section: features
            title-is-enough: true

commit 62db026b0ead7f0659df10c70e402c70ede5d7dd
Author: Cry Wolf <cry.wolf@centrum.cz>
Date:   Tue Jun 13 16:24:22 2023 +0200

    Added ability to skip commits.

    This allows commits to be skipped by typing 'changelog: skip'
    at the end of the commit. This is mainly useful for typo
    fixes or other things irrelevant to the user of a project.

    changelog:
        inherit: all
        section: features",
    );

    let git_cmd = Box::new(GitCmdMock::new(mocked_log));
    let git = Git::new(git_cmd);

    let f = File::open(YAML_FILE).unwrap();
    let template = Template::<changelog::Changes>::new(f).unwrap();
    let changelog = Changelog::new(template, git);

    let output = changelog.produce().unwrap();

    let exp_output = "\
============================================

## New features

* Support building on Debian Bookworm

### Allow configuring commit ID in yaml

This adds a field `skip-commits-up-to` into top level of yaml config so that users don't have to remember what to supply in `-c` argument every time.

### Added ability to skip commits.

This allows commits to be skipped by typing 'changelog: skip' \
at the end of the commit. This is mainly useful for typo fixes or other things irrelevant to the user of a project.

============================================";

    assert_eq!(exp_output, output);
}

#[test]
fn fails_when_unknown_section_in_commit() {
    let mocked_log = String::from(
        "\
commit 62db026b0ead7f0659df10c70e402c70ede5d7dd
Author: Cry Wolf <cry.wolf@centrum.cz>
Date:   Tue Jun 13 16:24:22 2023 +0200

    Added ability to skip commits.

    This allows commits to be skipped by typing 'changelog: skip'
    at the end of the commit. This is mainly useful for typo
    fixes or other things irrelevant to the user of a project.

    changelog:
        inherit: all
        section: unconfigured_section",
    );

    let git_cmd = Box::new(GitCmdMock::new(mocked_log));
    let git = Git::new(git_cmd);

    let f = File::open(YAML_FILE).unwrap();
    let template = Template::<changelog::Changes>::new(f).unwrap();
    let changelog = Changelog::new(template, git);

    let res = changelog.produce();

    assert!(res.is_err());
    assert!(res
        .unwrap_err()
        .to_string()
        .starts_with("Unknown section 'unconfigured_section' in changelog message"));
}

#[test]
fn fails_when_missing_section_key_in_commit() {
    let mocked_log = String::from(
        "\
commit 62db026b0ead7f0659df10c70e402c70ede5d7dd
Author: Cry Wolf <cry.wolf@centrum.cz>
Date:   Tue Jun 13 16:24:22 2023 +0200

    Added ability to skip commits.

    This allows commits to be skipped by typing 'changelog: skip'
    at the end of the commit. This is mainly useful for typo
    fixes or other things irrelevant to the user of a project.

    changelog:
        inherit: all",
    );

    let git_cmd = Box::new(GitCmdMock::new(mocked_log));
    let git = Git::new(git_cmd);

    let f = File::open(YAML_FILE).unwrap();
    let template = Template::<changelog::Changes>::new(f).unwrap();
    let changelog = Changelog::new(template, git);

    let res = changelog.produce();

    assert!(res.is_err());
    assert!(res
        .unwrap_err()
        .to_string()
        .starts_with("Missing 'section' key in changelog message:"));
}

#[test]
fn does_not_fail_when_only_one_line_in_commit() {
    let mocked_log = String::from(
        "\
commit b532ebcb0a214fbc69a5f5138e43eec14ea1a9dc
Author: Cry Wolf <cry.wolf@centrum.cz>
Date:   Tue Oct 24 19:17:09 2023 +0200

    Setup CI

    changelog:
        section: dev
        inherit: title
        title-is-enough: true

commit 62db026b0ead7f0659df10c70e402c70ede5d7dd
Author: Cry Wolf <cry.wolf@centrum.cz>
Date:   Tue Jun 13 16:24:22 2023 +0200

    Added ability to skip commits.

    changelog:
        inherit: all
        section: features",
    );

    let git_cmd = Box::new(GitCmdMock::new(mocked_log));
    let git = Git::new(git_cmd);

    let f = File::open(YAML_FILE).unwrap();
    let template = Template::<changelog::Changes>::new(f).unwrap();
    let changelog = Changelog::new(template, git);

    let output = changelog.produce().unwrap();

    let exp_output = "\
============================================

## New features

* Added ability to skip commits.

## Development

Internal development changes

* Setup CI

============================================";

    assert_eq!(exp_output, output);
}
