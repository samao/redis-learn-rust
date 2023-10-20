use crate::Connection;

#[derive(Debug)]
pub struct Unknown {
    command_name: String,
}

impl Unknown {
    pub(crate) fn new(key: impl ToString) -> Unknown {
        Unknown {
            command_name: key.to_string(),
        }
    }

    pub(crate) fn get_name(&self) -> &str {
        &self.command_name
    }

    pub(crate) async fn apply(self, dst: &mut Connection) -> crate::Result<()> {
        Ok(())
    }
}
