use std::path::Path;

pub trait ParentPath {
    fn parent_path(&self) -> &Path;
}

impl<T: AsRef<Path>> ParentPath for T {
    #[inline]
    #[allow(clippy::or_fun_call, reason = "Get a reference is cheap in this case")]
    fn parent_path(&self) -> &Path {
        self.as_ref().parent().unwrap_or(self.as_ref())
    }
}
