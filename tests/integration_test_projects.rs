mod mocks;

use mkchlog::changelog;
use mkchlog::changelog::Changelog;
use mkchlog::git::Git;
use mkchlog::template::Template;
use mocks::GitCmdMock;
use std::fs::File;

const YAML_FILE_PROJECTS: &str = "tests/mkchlog_projects.yml";
const YAML_FILE_SINCE_COMMIT: &str = "tests/mkchlog_projects_since_commit.yml";

#[test]
fn it_produces_correct_output_for_project1() {
    let mocked_log = String::from(
        "\
commit df841802133a1ad7556245bdce59417270de5c3f
Author: Martin Habovstiak <martin.habovstiak@gmail.com>
Date:   Sun Oct 25 10:12:50 2023 +0200

    Add configuration instructions to README.md

    The `fetch-depth` configuration isn't obvious for newbies so this
    documents it.

    changelog:
        project: mkchlog-action
        section: doc
        inherit: title
        title-is-enough: true

commit b532ebcb0a214fbc69a5f5138e43eec14ea1a9dc
Author: Cry Wolf <cry.wolf@centrum.cz>
Date:   Tue Oct 24 19:17:09 2023 +0200

    Setup CI

    changelog:
        project: mkchlog
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
        project: mkchlog

commit 11964cbb5ac05c5a19d75b5bebcc74ebc867e438
Author: Martin Habovstiak <martin.habovstiak@gmail.com>
Date:   Sun Oct 22 10:12:50 2023 +0200

    Publish release version rather than debug

    This updates the wasm module to one which was compiled with `--release`.

    changelog:
        project: mkchlog-action
        section: perf
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
        project: mkchlog
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
            project: mkchlog
            section: dev
            title-is-enough: true
",
    );

    let git_cmd = Box::new(GitCmdMock::new(mocked_log));
    let git = Git::new(git_cmd);

    let f = File::open(YAML_FILE_PROJECTS).unwrap();
    let mut template = Template::<changelog::Changes>::new(f).unwrap();
    let mut changelog = Changelog::new(&mut template, git);

    let project = Some("mkchlog".to_owned());
    let output = changelog.generate(project).unwrap();

    let exp_output = "\
============================================

## New features

* Support building on Debian Bookworm

### Allow configuring commit ID in yaml

This adds a field `skip-commits-up-to` into top level of yaml config so that users don't have to remember what to supply in `-c` argument every time.

## Development

Internal development changes

* Setup CI

* Setup Github Actions

============================================";

    assert_eq!(exp_output, output);
}

#[test]
fn it_produces_correct_output_for_project2() {
    let mocked_log = String::from(
        "\
commit df841802133a1ad7556245bdce59417270de5c3f
Author: Martin Habovstiak <martin.habovstiak@gmail.com>
Date:   Sun Oct 25 10:12:50 2023 +0200

    Add configuration instructions to README.md

    The `fetch-depth` configuration isn't obvious for newbies so this
    documents it.

    changelog:
        project: mkchlog-action
        section: doc
        inherit: title
        title-is-enough: true

commit b532ebcb0a214fbc69a5f5138e43eec14ea1a9dc
Author: Cry Wolf <cry.wolf@centrum.cz>
Date:   Tue Oct 24 19:17:09 2023 +0200

    Setup CI

    changelog:
        project: mkchlog
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
        project: mkchlog

commit 11964cbb5ac05c5a19d75b5bebcc74ebc867e438
Author: Martin Habovstiak <martin.habovstiak@gmail.com>
Date:   Sun Oct 22 10:12:50 2023 +0200

    Publish release version rather than debug

    This updates the wasm module to one which was compiled with `--release`.

    changelog:
        project: mkchlog-action
        section: perf
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
        project: mkchlog
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
            project: mkchlog
            section: dev
            title-is-enough: true
",
    );

    let git_cmd = Box::new(GitCmdMock::new(mocked_log));
    let git = Git::new(git_cmd);

    let f = File::open(YAML_FILE_PROJECTS).unwrap();
    let mut template = Template::<changelog::Changes>::new(f).unwrap();
    let mut changelog = Changelog::new(&mut template, git);

    let project = Some("mkchlog-action".to_owned());
    let output = changelog.generate(project).unwrap();

    let exp_output = "\
============================================

## Performance improvements

### Publish release version rather than debug

This updates the wasm module to one which was compiled with `--release`.

## Documentation changes

* Add configuration instructions to README.md

============================================";

    assert_eq!(exp_output, output);
}

#[test]
fn fails_when_called_with_incorrect_project_argument_provided() {
    let mocked_log = String::from("");

    let git_cmd = Box::new(GitCmdMock::new(mocked_log));
    let git = Git::new(git_cmd);

    let f = File::open(YAML_FILE_PROJECTS).unwrap();
    let mut template = Template::<changelog::Changes>::new(f).unwrap();
    let mut changelog = Changelog::new(&mut template, git);

    let project = Some("nonsense".to_owned());
    let res = changelog.generate(project);

    assert!(res.is_err());
    assert!(res
        .unwrap_err()
        .to_string()
        .starts_with("Project 'nonsense' not configured in config file"));
}

#[test]
fn it_produces_correct_output_for_project1_since_commit() {
    let mocked_log = String::from(
        "\
commit df841802133a1ad7556245bdce59417270de5c3f
Author: Martin Habovstiak <martin.habovstiak@gmail.com>
Date:   Sun Oct 25 10:12:50 2023 +0200

    Add configuration instructions to README.md

    The `fetch-depth` configuration isn't obvious for newbies so this
    documents it.

    changelog:
        project: mkchlog-action
        section: doc
        inherit: title
        title-is-enough: true

commit b532ebcb0a214fbc69a5f5138e43eec14ea1a9dc
Author: Cry Wolf <cry.wolf@centrum.cz>
Date:   Tue Oct 24 19:17:09 2023 +0200

    Setup CI

    changelog:
        project: mkchlog
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
        project: mkchlog

commit 11964cbb5ac05c5a19d75b5bebcc74ebc867e438
Author: Martin Habovstiak <martin.habovstiak@gmail.com>
Date:   Sun Oct 22 10:12:50 2023 +0200

    Publish release version rather than debug

    This updates the wasm module to one which was compiled with `--release`.

    changelog:
        project: mkchlog-action
        section: perf
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
",
    );

    let git_cmd = Box::new(GitCmdMock::new(mocked_log));
    let git = Git::new(git_cmd);

    let f = File::open(YAML_FILE_SINCE_COMMIT).unwrap();
    let mut template = Template::<changelog::Changes>::new(f).unwrap();
    let mut changelog = Changelog::new(&mut template, git);

    let project = Some("mkchlog".to_owned());
    let output = changelog.generate(project).unwrap();

    let exp_output = "\
============================================

## New features

* Support building on Debian Bookworm

### Allow configuring commit ID in yaml

This adds a field `skip-commits-up-to` into top level of yaml config so that users don't have to remember what to supply in `-c` argument every time.

## Development

Internal development changes

* Setup CI

* Setup Github Actions

============================================";

    assert_eq!(exp_output, output);
}

#[test]
fn it_produces_correct_output_for_project2_since_commit() {
    let mocked_log = String::from(
        "\
commit df841802133a1ad7556245bdce59417270de5c3f
Author: Martin Habovstiak <martin.habovstiak@gmail.com>
Date:   Sun Oct 25 10:12:50 2023 +0200

    Add configuration instructions to README.md

    The `fetch-depth` configuration isn't obvious for newbies so this
    documents it.

    changelog:
        project: mkchlog-action
        section: doc
        inherit: title
        title-is-enough: true

commit b532ebcb0a214fbc69a5f5138e43eec14ea1a9dc
Author: Cry Wolf <cry.wolf@centrum.cz>
Date:   Tue Oct 24 19:17:09 2023 +0200

    Setup CI

    changelog:
        project: mkchlog
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
        project: mkchlog

commit 11964cbb5ac05c5a19d75b5bebcc74ebc867e438
Author: Martin Habovstiak <martin.habovstiak@gmail.com>
Date:   Sun Oct 22 10:12:50 2023 +0200

    Publish release version rather than debug

    This updates the wasm module to one which was compiled with `--release`.

    changelog:
        project: mkchlog-action
        section: perf
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
",
    );

    let git_cmd = Box::new(GitCmdMock::new(mocked_log));
    let git = Git::new(git_cmd);

    let f = File::open(YAML_FILE_SINCE_COMMIT).unwrap();
    let mut template = Template::<changelog::Changes>::new(f).unwrap();
    let mut changelog = Changelog::new(&mut template, git);

    let project = Some("mkchlog-action".to_owned());
    let output = changelog.generate(project).unwrap();

    let exp_output = "\
============================================

## Performance improvements

### Publish release version rather than debug

This updates the wasm module to one which was compiled with `--release`.

## Documentation changes

* Add configuration instructions to README.md

============================================";

    assert_eq!(exp_output, output);
}
