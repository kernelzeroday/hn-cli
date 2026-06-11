use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct Item {
    pub id: u64,
    #[serde(rename = "type")]
    pub item_type: Option<String>,
    pub by: Option<String>,
    pub time: Option<u64>,
    pub title: Option<String>,
    pub url: Option<String>,
    pub text: Option<String>,
    pub score: Option<i64>,
    pub descendants: Option<u64>,
    pub parent: Option<u64>,
    pub kids: Option<Vec<u64>>,
    pub parts: Option<Vec<u64>>,
    pub dead: Option<bool>,
    pub deleted: Option<bool>,
}

#[derive(Deserialize, Debug)]
pub struct User {
    pub id: String,
    pub created: u64,
    pub karma: Option<i64>,
    pub about: Option<String>,
    pub submitted: Option<Vec<u64>>,
}

pub struct Client {
    http: reqwest::Client,
    base: String,
}

impl Client {
    pub fn new() -> Self {
        let http = reqwest::Client::builder()
            .user_agent("hn-cli/0.1")
            .build()
            .expect("Failed to create HTTP client");

        Self {
            http,
            base: "https://hacker-news.firebaseio.com/v0".to_string(),
        }
    }

    pub async fn get_item(&self, id: u64) -> Result<Item, reqwest::Error> {
        let url = format!("{}/item/{}.json", self.base, id);
        self.http.get(&url).send().await?.json().await
    }

    pub async fn get_user(&self, id: &str) -> Result<User, reqwest::Error> {
        let url = format!("{}/user/{}.json", self.base, id);
        self.http.get(&url).send().await?.json().await
    }

    pub async fn get_top_stories(&self) -> Result<Vec<u64>, reqwest::Error> {
        let url = format!("{}/topstories.json", self.base);
        self.http.get(&url).send().await?.json().await
    }

    pub async fn get_new_stories(&self) -> Result<Vec<u64>, reqwest::Error> {
        let url = format!("{}/newstories.json", self.base);
        self.http.get(&url).send().await?.json().await
    }

    pub async fn get_best_stories(&self) -> Result<Vec<u64>, reqwest::Error> {
        let url = format!("{}/beststories.json", self.base);
        self.http.get(&url).send().await?.json().await
    }

    pub async fn get_ask_stories(&self) -> Result<Vec<u64>, reqwest::Error> {
        let url = format!("{}/askstories.json", self.base);
        self.http.get(&url).send().await?.json().await
    }

    pub async fn get_show_stories(&self) -> Result<Vec<u64>, reqwest::Error> {
        let url = format!("{}/showstories.json", self.base);
        self.http.get(&url).send().await?.json().await
    }

    pub async fn get_job_stories(&self) -> Result<Vec<u64>, reqwest::Error> {
        let url = format!("{}/jobstories.json", self.base);
        self.http.get(&url).send().await?.json().await
    }

    pub async fn get_stories_with_details(
        &self,
        ids: &[u64],
        limit: usize,
    ) -> Result<Vec<Item>, reqwest::Error> {
        let ids_to_fetch = ids.iter().take(limit);
        let futures = ids_to_fetch.map(|id| self.get_item(*id));
        let results = futures::future::join_all(futures).await;

        let items: Vec<Item> = results
            .into_iter()
            .filter_map(|r| r.ok())
            .filter(|item| {
                matches!(item.item_type.as_deref(), Some("story") | Some("job"))
            })
            .filter(|item| item.dead != Some(true) && item.deleted != Some(true))
            .collect();

        Ok(items)
    }

    pub async fn get_comments(&self, kids: &[u64]) -> Result<Vec<Item>, reqwest::Error> {
        let futures = kids.iter().map(|id| self.get_item(*id));
        let results = futures::future::join_all(futures).await;

        let comments: Vec<Item> = results
            .into_iter()
            .filter_map(|r| r.ok())
            .filter(|item| item.dead != Some(true) && item.deleted != Some(true))
            .collect();

        Ok(comments)
    }

    pub async fn get_all_comments(&self, kids: &[u64]) -> Result<Vec<Item>, reqwest::Error> {
        let mut all = Vec::new();
        if kids.is_empty() {
            return Ok(all);
        }
        let comments = self.get_comments(kids).await?;
        let nested_kids: Vec<u64> = comments
            .iter()
            .filter_map(|c| c.kids.as_deref())
            .flatten()
            .copied()
            .collect();
        all.extend(comments);
        if !nested_kids.is_empty() {
            let nested = Box::pin(self.get_all_comments(&nested_kids)).await?;
            all.extend(nested);
        }
        Ok(all)
    }
}
