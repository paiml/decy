pub enum StdHeader {
    Assert,  // <assert.h>
    Ctype,   // <ctype.h>
    Errno,   // <errno.h>
    Float,   // <float.h>
    Limits,  // <limits.h>
    Locale,  // <locale.h>
    Math,    // <math.h>
    Setjmp,  // <setjmp.h>
    Signal,  // <signal.h>
    Stdarg,  // <stdarg.h>
    Stdbool, // <stdbool.h>
    Stddef,  // <stddef.h>
    Stdint,  // <stdint.h>
    Stdio,   // <stdio.h>
    Stdlib,  // <stdlib.h>
    String,  // <string.h>
    Time,    // <time.h>
    // POSIX headers
    Unistd,   // <unistd.h>
    Fcntl,    // <fcntl.h>
    Dirent,   // <dirent.h>
    SysTypes, // <sys/types.h>
    SysStat,  // <sys/stat.h>
    SysMman,  // <sys/mman.h>
    Wchar,    // <wchar.h>
}

    pub fn from_filename(filename: &str) -> Option<Self> {
        match filename {
            "assert.h" => Some(Self::Assert),
            "ctype.h" => Some(Self::Ctype),
            "errno.h" => Some(Self::Errno),
            "float.h" => Some(Self::Float),
            "limits.h" => Some(Self::Limits),
            "locale.h" => Some(Self::Locale),
            "math.h" => Some(Self::Math),
            "setjmp.h" => Some(Self::Setjmp),
            "signal.h" => Some(Self::Signal),
            "stdarg.h" => Some(Self::Stdarg),
            "stdbool.h" => Some(Self::Stdbool),
            "stddef.h" => Some(Self::Stddef),
            "stdint.h" => Some(Self::Stdint),
            "stdio.h" => Some(Self::Stdio),
            "stdlib.h" => Some(Self::Stdlib),
            "string.h" => Some(Self::String),
            "time.h" => Some(Self::Time),
            // POSIX headers
            "unistd.h" => Some(Self::Unistd),
            "fcntl.h" => Some(Self::Fcntl),
            "dirent.h" => Some(Self::Dirent),
            "sys/types.h" => Some(Self::SysTypes),
            "sys/stat.h" => Some(Self::SysStat),
            "sys/mman.h" => Some(Self::SysMman),
            "wchar.h" => Some(Self::Wchar),
            _ => None,
        }
    }

    pub fn len(&self) -> usize {
        self.functions.len()
    }

    pub fn is_empty(&self) -> bool {
        self.functions.is_empty()
    }

    fn default() -> Self {
        Self::new()
    }
