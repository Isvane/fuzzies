use fuzzies::{Dictionary as RustDictionary, SearchResult as RustSearchResult};
use napi::bindgen_prelude::*;
use napi_derive::napi;

#[napi(object)]
pub struct SearchResult {
  pub key: String,
  pub distance: u8,
}

impl From<RustSearchResult> for SearchResult {
  fn from(res: RustSearchResult) -> Self {
    Self {
      key: res.key,
      distance: res.distance,
    }
  }
}

#[napi(object)]
pub struct SearchOptions {
  pub limit: Option<u32>,
  pub distance: Option<u8>,
  pub transposition: Option<bool>,
  pub prefix: Option<bool>,
}

#[napi]
pub struct Dictionary {
  inner: RustDictionary,
}

#[napi]
impl Dictionary {
  #[napi(factory)]
  pub async fn open(path: String) -> Result<Self> {
    let inner = RustDictionary::open(&path)
      .map_err(|e| Error::from_reason(format!("Failed to open dictionary: {}", e)))?;
    Ok(Self { inner })
  }

  #[napi(factory)]
  pub fn from_array(keys: Vec<String>) -> Result<Self> {
    let inner = RustDictionary::from_iterator(keys)
      .map_err(|e| Error::from_reason(format!("Failed to create dictionary from array: {}", e)))?;
    Ok(Self { inner })
  }

  /// Compiles a byte-sorted text file into an immutable binary FST.
  #[napi]
  pub async fn build(input_path: String, output_path: String) -> Result<()> {
    RustDictionary::build(&input_path, &output_path)
      .map_err(|e| Error::from_reason(format!("Failed to build dictionary: {}", e)))?;
    Ok(())
  }

  /// Sorts a newline-delimited text file in-place by byte order.
  #[napi]
  pub async fn sort(path: String) -> Result<()> {
    RustDictionary::sort(&path)
      .map_err(|e| Error::from_reason(format!("Failed to sort file: {}", e)))?;
    Ok(())
  }

  #[napi]
  pub fn contains(&self, key: String) -> bool {
    self.inner.contains(key.as_bytes())
  }

  /// Returns the single top match for a query if it exists.
  #[napi]
  pub fn suggest(&self, query: String) -> Result<Option<SearchResult>> {
    let result = self
      .inner
      .suggest(&query)
      .map_err(|e| Error::from_reason(format!("Search failed: {}", e)))?;

    Ok(result.map(Into::into))
  }

  /// Executes a fuzzy search query.
  #[napi]
  pub fn search(&self, query: String, options: Option<SearchOptions>) -> Result<Vec<SearchResult>> {
    let mut builder = self.inner.search(&query);

    if let Some(opts) = options {
      if let Some(limit) = opts.limit {
        builder = builder.limit(limit as usize);
      }
      if let Some(dist) = opts.distance {
        builder = builder.distance(dist);
      }
      if let Some(trans) = opts.transposition {
        builder = builder.transposition(trans);
      }
      if let Some(pref) = opts.prefix {
        builder = builder.prefix(pref);
      }
    }

    let results = builder
      .execute()
      .map_err(|e| Error::from_reason(format!("Search failed: {}", e)))?;

    Ok(results.into_iter().map(Into::into).collect())
  }

  #[napi]
  pub fn len(&self) -> u32 {
    self.inner.len() as u32
  }
}
