use dlc_messages::oracle_msgs::OracleAnnouncement;

pub trait Storage {
    fn get_next_nonce_indexes(&self, num: usize) -> anyhow::Result<Vec<u32>>;

    fn save_announcement(
        &self,
        announcement: &OracleAnnouncement,
        indexes: Vec<u32>,
    ) -> anyhow::Result<()>;
}

impl Storage for () {
    fn get_next_nonce_indexes(&self, num: usize) -> anyhow::Result<Vec<u32>> {
        Ok((0..num as u32).collect())
    }

    fn save_announcement(
        &self,
        _announcement: &OracleAnnouncement,
        _indexes: Vec<u32>,
    ) -> anyhow::Result<()> {
        Ok(())
    }
}
