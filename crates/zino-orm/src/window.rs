use self::WindownFunction::*;
use super::{Entity, query::QueryExt};
use zino_core::model::Query;

/// A windown function.
#[derive(Debug, Clone, Copy)]
#[non_exhaustive]
enum WindownFunction<E: Entity> {
    /// The `COUNT` function.
    Count(E::Column),
    /// The `SUM` function.
    Sum(E::Column),
    /// The `AVG` function.
    Avg(E::Column),
    /// The `MIN` function.
    Min(E::Column),
    /// The `MAX` function.
    Max(E::Column),
    /// The `ROW_NUMBER` function.
    RowNumber,
    /// The `RNAK` function.
    Rank,
    /// The `DENSE_RNAK` function.
    DenseRank,
    /// The `PERCENT_RNAK` function.
    PercentRank,
    /// The `CUME_DIST` function.
    CumeDist,
    /// The `NTILE` function.
    Ntile(usize),
    /// The `LAG` function.
    Lag(E::Column, usize),
    /// The `LEAD` function.
    Lead(E::Column, usize),
    /// The `FIRST_VALUE` function.
    FirstValue(E::Column),
    /// The `LAST_VALUE` function.
    LastValue(E::Column),
    /// The `NTH_VALUE` function.
    NthValue(E::Column, usize),
}

/// SQL window functions.
///
/// # Examples
/// ```rust,ignore
/// use crate::model::{User, UserColumn::*};
/// use zino_orm::{QueryBuilder, Schema, Window};
///
/// let rank_window = Window::rank(CurrentLoginIp).order_desc(LoginCount);
/// let query = QueryBuilder::new()
///     .fields([Id, Name, CurrentLoginIp, LoginCount])
///     .window(rank_window, Some("login_count_rank"))
///     .and_not_in(Status, ["Deleted", "Locked"])
///     .order_desc(UpdatedAt)
///     .limit(10)
///     .build();
/// let users: Vec<Map> = User::find(&query).await?;
/// ```
#[derive(Debug, Clone, Copy)]
pub struct Window<E: Entity> {
    /// The window function.
    function: WindownFunction<E>,
    /// `PARTITION BY` a column.
    partition: E::Column,
    /// An optional `ORDER BY`.
    order: Option<(E::Column, bool)>,
}

impl<E: Entity> Window<E> {
    /// Constructs an instance for the window function `COUNT`.
    #[inline]
    pub fn count(col: E::Column, partition: E::Column) -> Self {
        Self {
            function: Count(col),
            partition,
            order: None,
        }
    }

    /// Constructs an instance for the window function `SUM`.
    #[inline]
    pub fn sum(col: E::Column, partition: E::Column) -> Self {
        Self {
            function: Sum(col),
            partition,
            order: None,
        }
    }

    /// Constructs an instance for the window function `AVG`.
    #[inline]
    pub fn avg(col: E::Column, partition: E::Column) -> Self {
        Self {
            function: Avg(col),
            partition,
            order: None,
        }
    }

    /// Constructs an instance for the window function `MIN`.
    #[inline]
    pub fn min(col: E::Column, partition: E::Column) -> Self {
        Self {
            function: Min(col),
            partition,
            order: None,
        }
    }

    /// Constructs an instance for the window function `MAX`.
    #[inline]
    pub fn max(col: E::Column, partition: E::Column) -> Self {
        Self {
            function: Max(col),
            partition,
            order: None,
        }
    }

    /// Constructs an instance for the window function `DENSE_RANK`.
    #[inline]
    pub fn row_number(partition: E::Column) -> Self {
        Self {
            function: RowNumber,
            partition,
            order: None,
        }
    }

    /// Constructs an instance for the window function `RANK`.
    #[inline]
    pub fn rank(partition: E::Column) -> Self {
        Self {
            function: Rank,
            partition,
            order: None,
        }
    }

    /// Constructs an instance for the window function `DENSE_RANK`.
    #[inline]
    pub fn dense_rank(partition: E::Column) -> Self {
        Self {
            function: DenseRank,
            partition,
            order: None,
        }
    }

    /// Constructs an instance for the window function `PERCENT_RANK`.
    #[inline]
    pub fn percent_rank(partition: E::Column) -> Self {
        Self {
            function: PercentRank,
            partition,
            order: None,
        }
    }

    /// Constructs an instance for the window function `CUME_DIST`.
    #[inline]
    pub fn cume_dist(partition: E::Column) -> Self {
        Self {
            function: CumeDist,
            partition,
            order: None,
        }
    }

    /// Constructs an instance for the window function `NTILE`.
    #[inline]
    pub fn ntile(num_buckets: usize, partition: E::Column) -> Self {
        Self {
            function: Ntile(num_buckets),
            partition,
            order: None,
        }
    }

    /// Constructs an instance for the window function `LAG`.
    #[inline]
    pub fn lag(col: E::Column, offset: usize, partition: E::Column) -> Self {
        Self {
            function: Lag(col, offset),
            partition,
            order: None,
        }
    }

    /// Constructs an instance for the window function `LEAD`.
    #[inline]
    pub fn lead(col: E::Column, offset: usize, partition: E::Column) -> Self {
        Self {
            function: Lead(col, offset),
            partition,
            order: None,
        }
    }

    /// Constructs an instance for the window function `FIRST_VALUE`.
    #[inline]
    pub fn first_value(col: E::Column, partition: E::Column) -> Self {
        Self {
            function: FirstValue(col),
            partition,
            order: None,
        }
    }

    /// Constructs an instance for the window function `LAST_VALUE`.
    #[inline]
    pub fn last_value(col: E::Column, partition: E::Column) -> Self {
        Self {
            function: LastValue(col),
            partition,
            order: None,
        }
    }

    /// Constructs an instance for the window function `NTH_VALUE`.
    #[inline]
    pub fn nth_value(col: E::Column, n: usize, partition: E::Column) -> Self {
        Self {
            function: NthValue(col, n),
            partition,
            order: None,
        }
    }

    /// Sets the sort order.
    #[inline]
    pub fn order_by(mut self, col: E::Column, descending: bool) -> Self {
        self.order = Some((col, descending));
        self
    }

    /// Sets the sort order with an ascending order.
    #[inline]
    pub fn order_asc(mut self, col: E::Column) -> Self {
        self.order = Some((col, false));
        self
    }

    /// Sets the sort order with an descending order.
    #[inline]
    pub fn order_desc(mut self, col: E::Column) -> Self {
        self.order = Some((col, true));
        self
    }

    /// Returns a default alias for the window function.
    pub(super) fn default_alias(&self) -> String {
        match &self.function {
            Count(col) => [col.as_ref(), "_sum"].concat(),
            Sum(col) => [col.as_ref(), "_sum"].concat(),
            Avg(col) => [col.as_ref(), "_avg"].concat(),
            Min(col) => [col.as_ref(), "_min"].concat(),
            Max(col) => [col.as_ref(), "_max"].concat(),
            RowNumber => "row_number".to_owned(),
            Rank => "rank".to_owned(),
            DenseRank => "dense_rank".to_owned(),
            PercentRank => "percent_rank".to_owned(),
            CumeDist => "cume_dist".to_owned(),
            Ntile(_) => "ntile".to_owned(),
            Lag(col, _) => [col.as_ref(), "_prev"].concat(),
            Lead(col, _) => [col.as_ref(), "_next"].concat(),
            FirstValue(col) => [col.as_ref(), "_first"].concat(),
            LastValue(col) => [col.as_ref(), "_last"].concat(),
            NthValue(col, _) => [col.as_ref(), "_nth"].concat(),
        }
    }

    /// Returns the SQL expression.
    pub(super) fn expr(&self) -> String {
        let partition_col_name = E::format_column(&self.partition);
        let partition = Query::format_field(&partition_col_name);
        let sort = self
            .order
            .as_ref()
            .map(|(col, descending)| {
                let col_name = E::format_column(col);
                let sort_field = Query::format_field(&col_name);
                if *descending {
                    format!(" ORDER BY {sort_field} DESC")
                } else {
                    format!(" ORDER BY {sort_field} ASC")
                }
            })
            .unwrap_or_default();
        match &self.function {
            Count(col) => {
                let col_name = E::format_column(col);
                let field = Query::format_field(&col_name);
                format!("count({field}) OVER (PARTITION BY {partition}{sort})")
            }
            Sum(col) => {
                let col_name = E::format_column(col);
                let field = Query::format_field(&col_name);
                format!("sum({field}) OVER (PARTITION BY {partition}{sort})")
            }
            Avg(col) => {
                let col_name = E::format_column(col);
                let field = Query::format_field(&col_name);
                format!("avg({field}) OVER (PARTITION BY {partition}{sort})")
            }
            Min(col) => {
                let col_name = E::format_column(col);
                let field = Query::format_field(&col_name);
                format!("min({field}) OVER (PARTITION BY {partition}{sort})")
            }
            Max(col) => {
                let col_name = E::format_column(col);
                let field = Query::format_field(&col_name);
                format!("max({field}) OVER (PARTITION BY {partition}{sort})")
            }
            RowNumber => {
                format!("row_number() OVER (PARTITION BY {partition}{sort})")
            }
            Rank => {
                format!("rank() OVER (PARTITION BY {partition}{sort})")
            }
            DenseRank => {
                format!("dense_rank() OVER (PARTITION BY {partition}{sort})")
            }
            PercentRank => {
                format!("percent_rank() OVER (PARTITION BY {partition}{sort})")
            }
            CumeDist => {
                format!("cume_dist() OVER (PARTITION BY {partition}{sort})")
            }
            Ntile(n) => {
                format!("ntile({n}) OVER (PARTITION BY {partition}{sort})")
            }
            Lag(col, offset) => {
                let col_name = E::format_column(col);
                let field = Query::format_field(&col_name);
                format!("lag({field}, {offset}) OVER (PARTITION BY {partition}{sort})")
            }
            Lead(col, offset) => {
                let col_name = E::format_column(col);
                let field = Query::format_field(&col_name);
                format!("lead({field}, {offset}) OVER (PARTITION BY {partition}{sort})")
            }
            FirstValue(col) => {
                let col_name = E::format_column(col);
                let field = Query::format_field(&col_name);
                format!("first_value({field}) OVER (PARTITION BY {partition}{sort})")
            }
            LastValue(col) => {
                let col_name = E::format_column(col);
                let field = Query::format_field(&col_name);
                format!("last_value({field}) OVER (PARTITION BY {partition}{sort})")
            }
            NthValue(col, n) => {
                let col_name = E::format_column(col);
                let field = Query::format_field(&col_name);
                format!("nth_value({field}, {n}) OVER (PARTITION BY {partition}{sort})")
            }
        }
    }
}
