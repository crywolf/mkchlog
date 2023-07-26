use mkchlog::git::GitLogCommand;
use std::error::Error;

pub struct GitCmdMock;

impl GitLogCommand for GitCmdMock {
    fn get_log(&self) -> Result<String, Box<dyn Error>> {
        let ouput = "\
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
        section: features";

        Ok(ouput.to_string())
    }
}
