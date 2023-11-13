# Mkchlog

Changelog generator tool suitable for user-facing changelogs and based on experiences of existing projects. More info on rationale is provided in the **Explanation** section below.

## Overview

### The mkchlog command

Program has these two subcommands:

* `check` - will verify the structure of commit messages (all commits or since the specified commit number, tj. since the last version/in the PR). Intended to be used in CI on PRs.
* `gen` - will process git history and output the changelog in markdown format into a file/stdout.

The command is configured using `.mkchlog.yml` - a per-project configuration file.
This is created when project is started (or added later) and rarely needs to be changed.
The main input is in commit messages.

#### Basic usage example

`mkchlog check`

`mkchlog gen`

You can provide additional options:
* Commit number to start. This one and previous commits will be skipped. By default, all commit messages are checked. (This option can be also specified in `.mkchlog.yml`.)
* Config (template) file name [default value is `.mkchlog.yml`]
* Path to the git repository [default value is the current directory]. (This option can be also specified in `.mkchlog.yml`.)
* `--from-stdin` Read commit(s) from stdin. Useful to use in a commit-msg git hook.

`mkchlog -c 7c85bee4303d56bededdfacf8fbb7bdc68e2195b -f .mkchlog.yml -g ../git-mkchlog-test/ gen`

Run `mkchlog help` for complete command options.

### Example inputs

#### Config file

```yaml
# .mkchlog.yml
# OPTIONAL Commit number to start (same as -c 7c85bee4303d56bededdfacf8fbb7bdc68e2195b)
skip-commits-up-to: 7c85bee4303d56bededdfacf8fbb7bdc68e2195b

# OPTIONAL Path to the git repository (same as -g ../git-mkchlog-test/)
git-path: ../git-mkchlog-test/

sections:
    # section identifier selected by project maintainer
    security:
        # The header presented to the user
        title: Security
        # desctiption is optional and will appear above changes
        description: This section contains very important security-related changes.
        subsections:
            vuln_fixes:
                title: Fixed vulnerabilities
    features:
        title: New features
    perf:
        title: Performance improvements
    dev:
        title: Development
        description: Internal development changes
```

#### Commits

```
Add ability to skip commits

This allows commits to be skipped by typing changelog: skip
at the end of the commit. This is mainly useful for typo
fixes or other things irrelevant to the user of a project.

changelog:
    inherit: all
    section: features
```

```
Fix grammar mistakes

We found 42 grammar mistakes that are fixed in this commit.

changelog: skip
```

```
Don't reallocate the buffer when we know its size

This computes the size and allocates the buffer upfront.
Avoiding allocations like this introduces 10% speedup.

changelog:
    section: perf
    title: Improve processing speed by 10%
    title-is-enough: true
```

```
Fix TOCTOU race condition when opening file

Previously we checked the file permissions before opening
the file now we check the metadata using file descriptor
after opening the file. (before reading)

changelog:
    section: security:vuln_fixes
    title: Fix vulnerability related to opening files
    description: The application was vulnerable to attacks
                 if the attacker had access to the working
                 directory. If you run this in such
                 enviroment you should update ASAP. If your
                 working directory is **not** accessible by
                 unprivileged users you don't need to worry.
```

```
Setup Github Actions

This configures github actions to test `mkchlog` as well as run it on
its own repository.

The new `.mkchlog.yml` is heavily inspired by the original example with
more sections, so we're more flexible in the future.

changelog:
	section: dev
	title-is-enough: true
```

```
Allow configuring commit ID in yaml

This adds a field `skip-commits-up-to` into top level of yaml config so that users don't have to remember what to supply in `-c` argument every time.

changelog:
    section: features
    inherit: all
```

### Example output

```markdown
## Security

This section contains very important security-related changes.

### Fixed vulnerabilities

#### Fix vulnerability related to opening files

The application was vulnerable to attacks if the attacker had access to the working directory. If you run this in such enviroment you should update ASAP. If your working directory is **not** accessible by unprivileged users you don't need to worry.

## New features

### Allow configuring commit ID in yaml

This adds a field `skip-commits-up-to` into top level of yaml config so that users don't have to remember what to supply in `-c` argument every time.

### Add ability to skip commits

This allows commits to be skipped by typing changelog: skip at the end of the commit. This is mainly useful for typo fixes or other things irrelevant to the user of a project.

## Performance improvements

* Improve processing speed by 10%

## Development

Internal development changes

* Setup Github Actions
```

## Multi-project setup

Some repositories host multiple projects that are related but should have disjoint changelogs. This is typical for multi-crate workspacess in Rust.

#### Config file

```yaml
# .mkchlog.yml
# top level
projects:
    list:  # list of allowed projects
    - main: [".", .github, .githooks] # name: [directory1, directory2, ...]
    - mkchlog: [mkchlog] # name: [directory]
    - mkchlog-action: [mkchlog-action] # name: [directory]

    since-commit: 11964cbb5ac05c5a19d75b5bebcc74ebc867e438 # projects are mandatory since COMMIT_NUMBER [optional]
    default: mkchlog # commits up to COMMIT_NUMBER are considered belonging to the project NAME [optional]
```

If projects list is provided then git commit must contain `project: x` in the changelog message where `x` is one of the specified project names. After a project name, the list of directories the project is contained in, must be provided. Project `main` in the example above is the "root" project that contains all files in the project root directory, plus other directories that do not belong to other projects.

To help with migration additional `since-commit` and `default` keywords can be used together. If they are specified then commits up to `since-commit` are considered belonging to `default` project.

#### Commits

```
Publish release version rather than debug

This updates the wasm module to one which was compiled with `--release`.

changelog:
    project: mkchlog-action
    section: perf
    inherit: all
```

#### Usage

To generate changelog for the `mkchlog-action` project use the following command:

`mkchlog --project mkchlog-action gen`

Run `mkchlog help` for complete command options.

## Git Hooks

To use locally configured githooks for the development you can run in the root directory of the repository:

`git config --local core.hooksPath .githooks/`

Alternatively add symlinks in your .git/hooks directory to any of the provided githooks.

## Explanation

(Rationale, idea and motivation by [Kixunil](https://github.com/Kixunil))

Commit messages should contain descriptive information about the change.
However not all of it is suitable to be in the changelog.
Each commit must be explicitly marked as either skipped or has some basic information filled.
Commits with `changelog: skip` will obviously not be included in the changelog.
Commits with `inherit: all` will simply include both title and description of the commit in the changelog.
This should be used when the commit message and description is equally useful for developers and users.
`inherit` could also accept additional options like `title` to only copy the title.
`section` is mandatory and defines in which section the change belongs.

`title` and `description` are those intended for the user.
The fictious "TOCTOU vulnerability fix" commit message above is hopefully a clear example.
For users it describes how it impacts them while for programmers it explains technical details of the issue.
`title-is-enough: true` explicitly opts-out of description, intended for situation when additional information is not needed for the user.

People refer to sections by their identifiers, not titles so that they don't accidentally duplicate the section just because of typo.
Unknown sections in commit messages are rejected.
Sections without commits are not present in the output at all so projects can have big templates without worrying about bloat.


This is based on these experiences:

* Editing a single file in PRs gets annoying with sufficient number of contributions
  because it causes frequent rebases with conflicts. Commit messages can not possibly cause conflicts.
* Sorting changes to categories later is difficult
  because it's hard to know if someone forgot to add a label on GitHub or the label is not appropriate.
  CI complaining about missing information prevents forgetting.

Additionally, we don't want to depend on GitHub so that we can migrate easily if needed - thus not pulling information from PRs.

---

### MSRV

The minimal supported Rust version is 1.63 - Debian Bookworm.
This also supports using the crates packaged in Debian - just delete `Cargo.lock` (which contains crates.io shasums of crates - some are slightly different in Debian).
