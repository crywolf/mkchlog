use mkchlog::changelog::Changelog;
use mkchlog::git::Git;
use mkchlog::template::Template;
use std::error::Error;

#[test]
fn it_produces_correct_output() -> Result<(), Box<dyn Error>> {
    let template = Template::new(".mkchlog.yml".to_owned())?;

    // TODO - mock Git
    let git = Git::new("../git-mkchlog-test/".to_owned());

    let changelog = Changelog::new(template, git);

    let output = changelog.produce()?;

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

### Added ability to skip commits.

This allows commits to be skipped by typing changelog: skip \
at the end of the commit. This is mainly useful for typo fixes or other things irrelevant to the user of a project.

## Performance improvements

* Improved processing speed by 10%
============================================";

    assert_eq!(exp_output, output);

    Ok(())
}
