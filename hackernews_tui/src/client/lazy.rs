use super::parser::*;
use crate::prelude::*;
use rayon::prelude::*;
use std::{sync::Arc, sync::RwLock};

/// `LazyLoadingComments` lazily loads comments on demand. It stores
/// a list of top comment's ids and a comment buffer. When more comments are needed,
/// the buffer is clear to load more comments. Additional comments are then
/// requested in background to reset the comment buffer.
pub struct LazyLoadingComments {
    client: client::client::HNClient,
    ids: Vec<u32>,
    comments: Arc<RwLock<Vec<Comment>>>,
}

impl LazyLoadingComments {
    pub fn new(client: client::HNClient, ids: Vec<u32>) -> Self {
        LazyLoadingComments {
            client,
            ids,
            comments: Arc::new(RwLock::new(vec![])),
        }
    }

    /// load all available comments in the current comment buffer then clear the buffer
    pub fn load_all(&self) -> Vec<Comment> {
        self.comments.write().unwrap().drain(..).collect::<Vec<_>>()
    }

    fn retrieve_comments_from_ids(
        client: client::HNClient,
        ids: Vec<u32>,
        comments: &Arc<RwLock<Vec<Comment>>>,
    ) {
        type ResultT = Vec<Result<Comment>>;

        let results: ResultT = ids
            .into_par_iter()
            .map(|id| {
                let response = client.get_item_from_id::<CommentResponse>(id)?;
                Ok(response.into())
            })
            .collect();

        let (oks, errs): (ResultT, ResultT) =
            results.into_iter().partition(|result| result.is_ok());

        errs.into_iter().for_each(|err| {
            warn!("failed to get comment: {:#?}", err);
        });

        let mut comments = comments.write().unwrap();
        oks.into_iter().for_each(|ok| {
            comments.push(ok.unwrap());
        });
    }

    /// drain the first `size` comment ids from the queue list,
    /// then request comments with the corresponding ids.
    /// parameter `block` determines whether the retrieving process should happen in background
    pub fn drain(&mut self, size: usize, block: bool) {
        if self.ids.is_empty() {
            return;
        }

        let ids: Vec<_> = self
            .ids
            .drain(0..std::cmp::min(self.ids.len(), size))
            .collect();

        let client = self.client.clone();
        if !block {
            let comments = Arc::clone(&self.comments);
            std::thread::spawn(move || Self::retrieve_comments_from_ids(client, ids, &comments));
        } else {
            Self::retrieve_comments_from_ids(client, ids, &self.comments);
        }
    }
}
