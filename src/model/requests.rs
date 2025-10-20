/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 19/10/25
******************************************************************************/

/// Parameters for getting recent prices (API v3)
#[derive(Debug, Clone, Default)]
pub struct RecentPricesRequest<'a> {
    /// Instrument epic
    pub epic: &'a str,
    /// Optional price resolution (default: MINUTE)
    pub resolution: Option<&'a str>,
    /// Optional start date time (yyyy-MM-dd'T'HH:mm:ss)
    pub from: Option<&'a str>,
    /// Optional end date time (yyyy-MM-dd'T'HH:mm:ss)
    pub to: Option<&'a str>,
    /// Optional max number of price points (default: 10)
    pub max_points: Option<i32>,
    /// Optional page size (default: 20, disable paging = 0)
    pub page_size: Option<i32>,
    /// Optional page number (default: 1)
    pub page_number: Option<i32>,
}

impl<'a> RecentPricesRequest<'a> {
    /// Create new parameters with just the epic (required field)
    pub fn new(epic: &'a str) -> Self {
        Self {
            epic,
            ..Default::default()
        }
    }

    /// Set the resolution
    pub fn with_resolution(mut self, resolution: &'a str) -> Self {
        self.resolution = Some(resolution);
        self
    }

    /// Set the from date
    pub fn with_from(mut self, from: &'a str) -> Self {
        self.from = Some(from);
        self
    }

    /// Set the to date
    pub fn with_to(mut self, to: &'a str) -> Self {
        self.to = Some(to);
        self
    }

    /// Set the max points
    pub fn with_max_points(mut self, max_points: i32) -> Self {
        self.max_points = Some(max_points);
        self
    }

    /// Set the page size
    pub fn with_page_size(mut self, page_size: i32) -> Self {
        self.page_size = Some(page_size);
        self
    }

    /// Set the page number
    pub fn with_page_number(mut self, page_number: i32) -> Self {
        self.page_number = Some(page_number);
        self
    }
}