// todo build new / -> Self

use std::fs;
use std::fs::{File, OpenOptions};
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::PathBuf;

use crate::storage::buffer::error::Error;
use crate::storage::buffer::wal_row::WalRow;
use crate::storage::file::encoding::Encoding;

const WAL_FILE_NAME: &str = ".wal";
// todo arc mutex ?
pub struct Wal {
    path: PathBuf,
    file: File,
    checkpoint: u64,
}

impl Wal {
    pub fn build(path: &str) -> Result<Wal, Error> {
        let path = PathBuf::from(path).join(WAL_FILE_NAME);
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(false)
            .open(&path)?;
        Ok(Wal {
            path,
            file,
            checkpoint: 0,
        })
    }

    pub fn write_transaction(&mut self, rows: &Vec<WalRow>) -> Result<(), Error> {
        for row in rows {
            self.file.write_all(&row.as_bytes()?)?;
            self.file.write_all(b"\n")?;
        }
        Ok(())
    }

    pub fn commit(&mut self) -> Result<(), Error> {
        Ok(self.file.flush()?)
    }

    pub fn read(&mut self) -> Result<Vec<WalRow>, Error> {
        let mut file_buffer: Vec<u8> = vec![];
        self.file.seek(SeekFrom::Start(self.checkpoint))?;
        self.file.read_to_end(&mut file_buffer)?;
        if file_buffer.is_empty() {
            Ok(vec![])
        } else {
            let rows: Vec<WalRow> = file_buffer[..file_buffer.len() - 1]
                .split(|&b| b == b'\n')
                .map(|bytes| WalRow::from_bytes(bytes).unwrap())
                .collect();
            self.checkpoint = self.file.stream_position()?;
            Ok(rows)
        }
    }

    pub fn vacuum(&mut self) -> Result<(), Error> {
        self.commit()?;
        let rows = self.read()?;
        fs::remove_file(&self.path)?;
        self.file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(false)
            .open(&self.path)?;
        self.write_transaction(&rows)?;
        self.checkpoint = 0;
        Ok(())
    }
}

#[cfg(test)]
pub mod tests {
    use crate::storage::buffer::wal::Wal;
    use crate::storage::buffer::wal_row::Operation;
    use crate::storage::buffer::wal_row::tests::get_test_wal_row;
    use crate::storage::tests::{delete_test_env, init_test_env};

    const TEST_PATH: &str = "target/tests/wal";

    #[test]
    fn write_transaction_should_log_in_file() {
        let path = init_test_env(TEST_PATH, "write_transaction");
        let mut wal = Wal::build(path.to_str().unwrap()).unwrap();
        wal.write_transaction(&vec![get_test_wal_row(), get_test_wal_row()])
            .unwrap();
        wal.write_transaction(&vec![get_test_wal_row()]).unwrap();
        wal.commit().unwrap();
        let rows = wal.read().unwrap();
        assert_eq!(rows.len(), 3);
        for row in rows {
            assert_eq!(row.transaction_id, 23);
            assert_eq!(row.transaction_size, 66);
            assert_eq!(row.operation, Operation::Insert);
        }
        delete_test_env(TEST_PATH, "write_transaction");
    }

    #[test]
    fn read_should_rows_from_begin() {
        let path = init_test_env(TEST_PATH, "read_01");
        let mut wal = Wal::build(path.to_str().unwrap()).unwrap();
        let rows = vec![get_test_wal_row(), get_test_wal_row(), get_test_wal_row()];
        wal.write_transaction(&rows).unwrap();
        wal.commit().unwrap();
        let rows = wal.read().unwrap();
        assert_eq!(rows.len(), 3);
        for row in rows {
            assert_eq!(row.transaction_id, 23);
            assert_eq!(row.transaction_size, 66);
            assert_eq!(row.operation, Operation::Insert);
        }
        assert_eq!(wal.checkpoint, 270);
        delete_test_env(TEST_PATH, "read_01");
    }

    #[test]
    fn read_should_rows_from_checkpoint() {
        let path = init_test_env(TEST_PATH, "read_02");
        let mut wal = Wal::build(path.to_str().unwrap()).unwrap();
        let rows = vec![get_test_wal_row(), get_test_wal_row(), get_test_wal_row()];
        wal.write_transaction(&rows).unwrap();
        wal.commit().unwrap();
        wal.checkpoint = 90;
        let rows = wal.read().unwrap();
        assert_eq!(rows.len(), 2);
        for row in rows {
            assert_eq!(row.transaction_id, 23);
            assert_eq!(row.transaction_size, 66);
            assert_eq!(row.operation, Operation::Insert);
        }
        assert_eq!(wal.checkpoint, 270);
        delete_test_env(TEST_PATH, "read_02");
    }

    #[test]
    fn vacuum_should_truncate_from_offset() {
        let path = init_test_env(TEST_PATH, "vacuum");
        let mut wal = Wal::build(path.to_str().unwrap()).unwrap();
        let rows = vec![get_test_wal_row(), get_test_wal_row(), get_test_wal_row()];
        wal.write_transaction(&rows).unwrap();
        wal.commit().unwrap();
        wal.read().unwrap();
        let rows = vec![get_test_wal_row()];
        wal.write_transaction(&rows).unwrap();
        wal.vacuum().unwrap();
        assert_eq!(wal.read().unwrap(), rows);
        delete_test_env(TEST_PATH, "vacuum");
    }
}
