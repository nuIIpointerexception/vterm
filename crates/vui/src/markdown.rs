use std::fmt;

pub struct MdList<'data, T>(pub &'data [T]);

impl<'data, T> fmt::Debug for MdList<'data, T>
    where
        T: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("\n")?;
        for entry in self.0 {
            if f.alternate() {
                f.write_fmt(format_args!("- {:#?}\n", entry))?;
            } else {
                f.write_fmt(format_args!("- {:?}\n", entry))?;
            }
        }
        Ok(())
    }
}

impl<'data, T> fmt::Display for MdList<'data, T>
    where
        T: fmt::Display,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("\n")?;
        for entry in self.0 {
            f.write_fmt(format_args!("- {}\n", entry))?;
        }
        Ok(())
    }
}
