use std::collections::{BTreeMap, HashMap};
use std::ffi::OsStr;
use std::fs::{self, File, OpenOptions};
use std::io::{self, BufReader, BufWriter, Read, Seek, SeekFrom, Write};
use std::ops::Range;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use serde_json::Deserializer;

use super::KvsEngine;

use crate::KvError;

const COMPACTION_THRESHOLD: u64 = 1024 * 1024;

pub struct KvStore {
    path: PathBuf,

    readers: HashMap<u64, BufReaderWithPos<File>>,

    writer: BufWriterWithPos<File>,

    index: BTreeMap<String, CommandPos>,

    current_gen: u64,

    uncompacted: u64,
}

impl KvStore {
    pub fn open<T>(path: T) -> Result<KvStore, KvError>
    where
        T: Into<PathBuf>,
    {
        let path = path.into();

        fs::create_dir_all(&path)?;

        let mut readers = HashMap::new();
        let mut index = BTreeMap::new();

        let gen_list = sorted_gen_list(&path)?;

        let mut uncompacted = 0;

        for &gen in &gen_list {
            let mut reader = BufReaderWithPos::new(File::open(log_path(&path, gen))?)?;
            uncompacted += load(gen, &mut index, &mut reader)?;
            readers.insert(gen, reader);
        }

        let current_gen = gen_list.last().unwrap_or(&1);

        let writer = new_log_file(&path, *current_gen, &mut readers)?;

        Ok(KvStore {
            path: path,
            readers: readers,
            writer: writer,
            current_gen: *current_gen,
            index: index,
            uncompacted: uncompacted,
        })
    }

    fn compact(&mut self) -> Result<(), KvError> {
        let compaction_gen = self.current_gen + 1;
        self.current_gen += 2;
        self.writer = self.new_log_file(self.current_gen)?;

        let mut compaction_writer = self.new_log_file(compaction_gen)?;

        let mut new_pos = 0; // pos in the new log file.
        for cmd_pos in &mut self.index.values_mut() {
            let reader = self
                .readers
                .get_mut(&cmd_pos.gen)
                .expect("Cannot find log reader");
            if reader.pos != cmd_pos.pos {
                reader.seek(SeekFrom::Start(cmd_pos.pos))?;
            }

            let mut entry_reader = reader.take(cmd_pos.len);
            let len = io::copy(&mut entry_reader, &mut compaction_writer)?;
            *cmd_pos = (compaction_gen, new_pos..new_pos + len).into();
            new_pos += len;
        }
        compaction_writer.flush()?;

        // remove stale log files.
        let stale_gens: Vec<_> = self
            .readers
            .keys()
            .filter(|&&gen| gen < compaction_gen)
            .cloned()
            .collect();

        for stale_gen in stale_gens {
            self.readers.remove(&stale_gen);
            fs::remove_file(log_path(&self.path, stale_gen))?;
        }

        self.uncompacted = 0;

        Ok(())
    }

    fn new_log_file(&mut self, gen: u64) -> Result<BufWriterWithPos<File>, KvError> {
        new_log_file(&self.path, gen, &mut self.readers)
    }
}

impl KvsEngine for KvStore {
    fn set(&mut self, key: String, value: String) -> Result<(), KvError> {
        let cmd = Command::set(key, value);
        let pos = self.writer.pos;
        serde_json::to_writer(&mut self.writer, &cmd)?;
        self.writer.flush()?;
        if let Command::Set { key, value: _ } = cmd {
            if let Some(old_cmd) = self
                .index
                .insert(key, (self.current_gen, pos..self.writer.pos).into())
            {
                self.uncompacted += old_cmd.len;
            }
        }

        if self.uncompacted > COMPACTION_THRESHOLD {
            self.compact()?;
        }

        Ok(())
    }

    fn get(&mut self, key: String) -> Result<Option<String>, KvError> {
        if let Some(cmd_pods) = self.index.get(&key) {
            let reader = self
                .readers
                .get_mut(&cmd_pods.gen)
                .expect("Cannot find log reader");
            reader.seek(SeekFrom::Start(cmd_pods.pos))?;
            let cmd_reader = reader.take(cmd_pods.len);

            if let Command::Set { key: _, value } = serde_json::from_reader(cmd_reader)? {
                Ok(Some(value))
            } else {
                Err(KvError::UnexpectedCommandType)
            }
        } else {
            Ok(None)
        }
    }

    fn remove(&mut self, key: String) -> Result<(), KvError> {
        if self.index.contains_key(&key) {
            let cmd = Command::remove(key);
            serde_json::to_writer(&mut self.writer, &cmd)?;
            self.writer.flush()?;
            if let Command::Remove { key } = cmd {
                let old_cmd = self.index.remove(&key).expect("key not found");
                self.uncompacted += old_cmd.len;
            }

            Ok(())
        } else {
            Err(KvError::KeyNotFound)
        }
    }
}

/// Struct representing a command.
#[derive(Serialize, Deserialize, Debug)]
enum Command {
    Set { key: String, value: String },
    Remove { key: String },
}

impl Command {
    fn set(key: String, value: String) -> Command {
        Command::Set { key, value }
    }

    fn remove(key: String) -> Command {
        Command::Remove { key }
    }
}

/// ?????????????????????????????????
struct CommandPos {
    gen: u64,
    pos: u64,
    len: u64,
}

impl From<(u64, Range<u64>)> for CommandPos {
    fn from((gen, range): (u64, Range<u64>)) -> Self {
        CommandPos {
            gen: gen,
            pos: range.start,
            len: range.end - range.start,
        }
    }
}

struct BufReaderWithPos<T>
where
    T: Seek + Read,
{
    reader: BufReader<T>,
    pos: u64,
}

impl<T> BufReaderWithPos<T>
where
    T: Seek + Read,
{
    fn new(mut inner: T) -> Result<Self, KvError> {
        let pos = inner.seek(SeekFrom::Current(0))?;

        Ok(Self {
            reader: BufReader::new(inner),
            pos: pos,
        })
    }
}

impl<T> Read for BufReaderWithPos<T>
where
    T: Seek + Read,
{
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let len = self.reader.read(buf)?;
        self.pos += len as u64;
        Ok(len)
    }
}
impl<T> Seek for BufReaderWithPos<T>
where
    T: Seek + Read,
{
    fn seek(&mut self, pos: SeekFrom) -> io::Result<u64> {
        self.pos = self.reader.seek(pos)?;
        Ok(self.pos)
    }
}

struct BufWriterWithPos<T>
where
    T: Seek + Write,
{
    writer: BufWriter<T>,
    pos: u64,
}

impl<T> BufWriterWithPos<T>
where
    T: Seek + Write,
{
    fn new(mut inner: T) -> Result<Self, KvError> {
        let pos = inner.seek(SeekFrom::Current(0))?;
        Ok(Self {
            writer: BufWriter::new(inner),
            pos: pos,
        })
    }
}

impl<T> Seek for BufWriterWithPos<T>
where
    T: Seek + Write,
{
    fn seek(&mut self, pos: SeekFrom) -> io::Result<u64> {
        self.pos = self.writer.seek(pos)?;
        Ok(self.pos)
    }
}

impl<T> Write for BufWriterWithPos<T>
where
    T: Seek + Write,
{
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let len = self.writer.write(buf)?;
        self.pos += len as u64;
        Ok(len)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.writer.flush()
    }
}

fn log_path(dir: &Path, gen: u64) -> PathBuf {
    dir.join(format!("{}.log", gen))
}

fn load(
    gen: u64,
    index: &mut BTreeMap<String, CommandPos>,
    reader: &mut BufReaderWithPos<File>,
) -> Result<u64, KvError> {
    let mut pos = reader.seek(SeekFrom::Start(0))?;
    let mut stream = Deserializer::from_reader(reader).into_iter::<Command>();
    let mut uncompacted = 0;

    while let Some(cmd) = stream.next() {
        let new_pos = stream.byte_offset() as u64;
        match cmd? {
            Command::Set { key, value: _ } => {
                if let Some(old_cmd) = index.insert(key, (gen, pos..new_pos).into()) {
                    uncompacted += old_cmd.len;
                }
            }
            Command::Remove { key } => {
                if let Some(old_cmd) = index.remove(&key) {
                    uncompacted += old_cmd.len;
                }
                uncompacted += new_pos - pos;
            }
        }
        pos = new_pos
    }

    Ok(uncompacted)
}

fn sorted_gen_list(path: &Path) -> Result<Vec<u64>, KvError> {
    let mut gen_list: Vec<u64> = fs::read_dir(&path)?
        .flat_map(|res| -> Result<PathBuf, KvError> { Ok(res?.path()) })
        .filter(|path| path.is_file() && path.extension() == Some("log".as_ref()))
        .flat_map(|path| {
            path.file_name()
                .and_then(OsStr::to_str)
                .map(|s| s.trim_end_matches(".log"))
                .map(str::parse::<u64>)
        })
        .flatten()
        .collect();
    gen_list.sort_unstable();

    Ok(gen_list)
}

fn new_log_file(
    path: &Path,
    gen: u64,
    readers: &mut HashMap<u64, BufReaderWithPos<File>>,
) -> Result<BufWriterWithPos<File>, KvError> {
    let path = log_path(&path, gen);
    let writer = BufWriterWithPos::new(
        OpenOptions::new()
            .create(true)
            .write(true)
            .append(true)
            .open(&path)?,
    )?;

    readers.insert(gen, BufReaderWithPos::new(File::open(&path)?)?);

    Ok(writer)
}
