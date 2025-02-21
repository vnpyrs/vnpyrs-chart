use std::io::{BufReader, Read};
use std::sync::LazyLock;

use chrono::{DateTime, Local, NaiveDateTime};
use glam::Vec4;

use crate::vertex::{CandleVertex, TradePairVertex, Vertex, VolumeVertex};

pub const MIN_BAR_COUNT: i64 = 50;

fn get_home_path() -> String {
    let win = std::env::var("USERPROFILE");
    let unix = std::env::var("HOME");
    if win.is_ok() {
        win.unwrap()
    } else if unix.is_ok() {
        unix.unwrap()
    } else {
        ".".to_string()
    }
}

#[derive(Default)]
pub struct Manager {
    pub left_ix: i64,
    pub right_ix: i64,
    pub min_price_view: f64,
    pub max_price_view: f64,
    pub max_volume_view: f64,
    pub cursor_ix: i64,
    pub cursor_price: Option<f64>,
    pub cursor_volume: Option<f64>,
    pub current_cursor_position: (f64, f64),
    pub pressed_position: Option<(f64, f64)>,
    pub pressed_left_right_ix: Option<(i64, i64)>,
}

impl Manager {
    pub fn new() -> Self {
        if HISTORY.high_price.is_empty() {
            panic!("没有数据");
        }
        let right_ix = HISTORY.high_price.len() as i64 - 1;
        Manager {
            right_ix,
            ..Default::default()
        }
    }

    pub fn update_maxmin_by_left_right_ix(&mut self) {
        (self.min_price_view, self.max_price_view) = get_price_range(self.left_ix, self.right_ix);
        self.max_volume_view = get_volume_max(self.left_ix, self.right_ix);
    }

    //放大
    pub fn zoom_in(&mut self) {
        if self.right_ix - self.left_ix < MIN_BAR_COUNT {
            return;
        }
        let bar_count = self.right_ix - self.left_ix + 1;
        self.left_ix += (bar_count as f32 * 0.2) as i64;
    }

    //缩小
    pub fn zoom_out(&mut self) {
        let bar_count = self.right_ix - self.left_ix + 1;
        self.left_ix -= (bar_count as f32 * 0.25) as i64;
        self.left_ix = self.left_ix.max(0);
    }

    //以cursor_ix为轴放大
    pub fn zoom_in_by(&mut self) {
        if self.right_ix - self.left_ix < MIN_BAR_COUNT {
            return;
        }
        let bar_count_left = self.cursor_ix - self.left_ix + 1;
        self.left_ix += (bar_count_left as f32 * 0.2) as i64;
        let bar_count_right = self.right_ix - self.cursor_ix + 1;
        self.right_ix -= (bar_count_right as f32 * 0.2) as i64;
    }

    //以cursor_ix为轴缩小
    pub fn zoom_out_by(&mut self) {
        let bar_count_left = self.cursor_ix - self.left_ix + 1;
        self.left_ix -= (bar_count_left as f32 * 0.25) as i64;
        self.left_ix = self.left_ix.max(0);
        let bar_count_right = self.right_ix - self.cursor_ix + 1;
        self.right_ix += (bar_count_right as f32 * 0.25) as i64;
        self.right_ix = self.right_ix.min(HISTORY.datetime.len() as i64 - 1);
    }
}

pub fn get_price_range(left_ix: i64, right_ix: i64) -> (f64, f64) {
    let max_price = *HISTORY.high_price[left_ix as usize..=right_ix as usize]
        .iter()
        .max_by(|x, y| x.partial_cmp(y).unwrap())
        .unwrap();
    let min_price = *HISTORY.low_price[left_ix as usize..=right_ix as usize]
        .iter()
        .min_by(|x, y| x.partial_cmp(y).unwrap())
        .unwrap();
    (min_price, max_price)
}

pub fn get_volume_max(left_ix: i64, right_ix: i64) -> f64 {
    *HISTORY.volume[left_ix as usize..=right_ix as usize]
        .iter()
        .max_by(|x, y| x.partial_cmp(y).unwrap())
        .unwrap()
}

pub struct HistoryData {
    // pub timestamp: Vec<u64>,
    pub datetime: Vec<NaiveDateTime>,
    pub open_price: Vec<f64>,
    pub high_price: Vec<f64>,
    pub low_price: Vec<f64>,
    pub close_price: Vec<f64>,
    pub volume: Vec<f64>,
}
pub static HISTORY: LazyLock<HistoryData> = LazyLock::new(|| {
    let home_path = get_home_path();
    let mut reader = BufReader::new(
        std::fs::File::open(format!("{}/vnpyrs/history.dat", home_path))
            .expect(&format!("打开文件{}/vnpyrs/history.dat失败", home_path)),
    );
    let mut buf = [0u8; 8];
    let error_string = format!("读取文件{}/vnpyrs/history.dat失败", home_path);
    reader.read_exact(&mut buf).expect(&error_string); //读取版本号
    if u64::from_le_bytes(buf) != 0 {
        panic!("请升级版本");
    }
    reader.read_exact(&mut buf).expect(&error_string);
    let count = u64::from_le_bytes(buf);
    // let mut timestamp: Vec<u64> = Vec::with_capacity(count as usize);
    let mut datetime: Vec<NaiveDateTime> = Vec::with_capacity(count as usize);
    let mut open_price: Vec<f64> = Vec::with_capacity(count as usize);
    let mut high_price: Vec<f64> = Vec::with_capacity(count as usize);
    let mut low_price: Vec<f64> = Vec::with_capacity(count as usize);
    let mut close_price: Vec<f64> = Vec::with_capacity(count as usize);
    let mut volume: Vec<f64> = Vec::with_capacity(count as usize);
    for _ in 0..count {
        reader.read_exact(&mut buf).expect(&error_string);
        let ts = u64::from_le_bytes(buf);
        // timestamp.push(ts);
        let local_datetime: DateTime<Local> =
            DateTime::from_timestamp(ts as i64, 0).unwrap().into();
        datetime.push(local_datetime.naive_local());

        reader.read_exact(&mut buf).expect(&error_string);
        open_price.push(f64::from_le_bytes(buf));

        reader.read_exact(&mut buf).expect(&error_string);
        high_price.push(f64::from_le_bytes(buf));

        reader.read_exact(&mut buf).expect(&error_string);
        low_price.push(f64::from_le_bytes(buf));

        reader.read_exact(&mut buf).expect(&error_string);
        close_price.push(f64::from_le_bytes(buf));

        reader.read_exact(&mut buf).expect(&error_string);
        volume.push(f64::from_le_bytes(buf));
    }
    HistoryData {
        // timestamp,
        datetime,
        open_price,
        high_price,
        low_price,
        close_price,
        volume,
    }
});

const LONG: u8 = 1;
const SHORT: u8 = 2;

#[derive(Clone, Copy)]
pub struct TradeData {
    // pub timestamp: u64,
    pub datetime: NaiveDateTime,
    pub direction: u8,
    pub price: f64,
    pub volume: f64,
}

pub static TRADES: LazyLock<Vec<TradeData>> = LazyLock::new(|| {
    let mut trades = Vec::new();
    let home_path = get_home_path();
    let mut reader = BufReader::new(
        std::fs::File::open(format!("{}/vnpyrs/trades.dat", home_path))
            .expect(&format!("打开文件{}/vnpyrs/trades.dat失败", home_path)),
    );
    let mut buf = [0u8; 8];
    let mut buf1 = [0u8; 1];
    let error_string = format!("读取文件{}/vnpyrs/trades.dat失败", home_path);
    reader.read_exact(&mut buf).expect(&error_string); //读取版本号
    if u64::from_le_bytes(buf) != 0 {
        panic!("请升级版本");
    }
    reader.read_exact(&mut buf).expect(&error_string);
    let count = u64::from_le_bytes(buf);
    for _ in 0..count {
        reader.read_exact(&mut buf).expect(&error_string);
        let timestamp = u64::from_le_bytes(buf);
        let local_datetime: DateTime<Local> = DateTime::from_timestamp(timestamp as i64, 0)
            .unwrap()
            .into();
        let datetime = local_datetime.naive_local();

        reader.read_exact(&mut buf1).expect(&error_string);
        let direction = u8::from_le_bytes(buf1);

        reader.read_exact(&mut buf).expect(&error_string);
        let price = f64::from_le_bytes(buf);

        reader.read_exact(&mut buf).expect(&error_string);
        let volume = f64::from_le_bytes(buf);

        trades.push(TradeData {
            // timestamp,
            datetime,
            direction,
            price,
            volume,
        });
    }
    trades
});

pub struct TradePair {
    pub open_dt: NaiveDateTime,
    pub open_price: f64,
    pub close_dt: NaiveDateTime,
    pub close_price: f64,
    pub direction: u8,
    pub volume: f64,
}

pub const MY_EPSILON: f64 = 0.0000000001;

pub fn generate_trade_pairs() -> Vec<TradePair> {
    let mut long_trades: Vec<TradeData> = Vec::new();
    let mut short_trades: Vec<TradeData> = Vec::new();
    let mut trade_pairs: Vec<TradePair> = Vec::new();

    for trade in TRADES.iter() {
        let mut trade = trade.clone();

        let same_direction: &mut Vec<TradeData>;
        let opposite_direction: &mut Vec<TradeData>;
        if trade.direction == LONG {
            same_direction = &mut long_trades;
            opposite_direction = &mut short_trades;
        } else {
            same_direction = &mut short_trades;
            opposite_direction = &mut long_trades;
        }
        while trade.volume.abs() > MY_EPSILON && !opposite_direction.is_empty() {
            let mut open_trade = opposite_direction[0];

            let close_volume = f64::min(open_trade.volume, trade.volume);
            let d = TradePair {
                open_dt: open_trade.datetime,
                open_price: open_trade.price,
                close_dt: trade.datetime,
                close_price: trade.price,
                direction: open_trade.direction,
                volume: close_volume,
            };
            trade_pairs.push(d);

            open_trade.volume -= close_volume;
            if open_trade.volume.abs() < MY_EPSILON {
                opposite_direction.pop();
            }

            trade.volume -= close_volume;
        }

        if trade.volume.abs() > MY_EPSILON {
            same_direction.push(trade);
        }
    }
    trade_pairs
}

pub static TRADE_PAIRS: LazyLock<Vec<TradePair>> = LazyLock::new(|| generate_trade_pairs());

pub static CANDLE_VERTEX: LazyLock<CandleVertex> = LazyLock::new(|| {
    let mut open_iter = HISTORY.open_price.iter();
    let mut high_iter = HISTORY.high_price.iter();
    let mut low_iter = HISTORY.low_price.iter();
    let mut close_iter = HISTORY.close_price.iter();
    let mut up = Vec::new();
    let mut down = Vec::new();
    let mut down_hl = Vec::new();
    let mut stay = Vec::new();
    let mut i = 0f32;
    while let Some(open_price) = open_iter.next() {
        let high_price = high_iter.next().unwrap();
        let low_price = low_iter.next().unwrap();
        let close_price = close_iter.next().unwrap();
        if close_price > open_price {
            up.extend(&[
                Vertex {
                    position: [i - 0.4, *close_price as f32],
                },
                Vertex {
                    position: [i + 0.4, *close_price as f32],
                },
                Vertex {
                    position: [i + 0.4, *close_price as f32],
                },
                Vertex {
                    position: [i + 0.4, *open_price as f32],
                },
                Vertex {
                    position: [i + 0.4, *open_price as f32],
                },
                Vertex {
                    position: [i - 0.4, *open_price as f32],
                },
                Vertex {
                    position: [i - 0.4, *open_price as f32],
                },
                Vertex {
                    position: [i - 0.4, *close_price as f32],
                },
                Vertex {
                    position: [i, *high_price as f32],
                },
                Vertex {
                    position: [i, *close_price as f32],
                },
                Vertex {
                    position: [i, *low_price as f32],
                },
                Vertex {
                    position: [i, *open_price as f32],
                },
            ]);
        } else if close_price < open_price {
            down.extend(&[
                Vertex {
                    position: [i - 0.4, *open_price as f32],
                },
                Vertex {
                    position: [i - 0.4, *close_price as f32],
                },
                Vertex {
                    position: [i + 0.4, *open_price as f32],
                },
                Vertex {
                    position: [i + 0.4, *open_price as f32],
                },
                Vertex {
                    position: [i - 0.4, *close_price as f32],
                },
                Vertex {
                    position: [i + 0.4, *close_price as f32],
                },
            ]);
            down_hl.extend(&[
                Vertex {
                    position: [i, *high_price as f32],
                },
                Vertex {
                    position: [i, *low_price as f32],
                },
            ]);
        } else {
            stay.extend(&[
                Vertex {
                    position: [i - 0.4, *open_price as f32],
                },
                Vertex {
                    position: [i + 0.4, *close_price as f32],
                },
                Vertex {
                    position: [i, *high_price as f32],
                },
                Vertex {
                    position: [i, *low_price as f32],
                },
            ]);
        }
        i += 1.0;
    }
    CandleVertex {
        up,
        down,
        down_hl,
        stay,
    }
});

pub static VOLUME_VERTEX: LazyLock<VolumeVertex> = LazyLock::new(|| {
    let mut open_iter = HISTORY.open_price.iter();
    let mut close_iter = HISTORY.close_price.iter();
    let mut volume_iter = HISTORY.volume.iter();
    let mut up = Vec::new();
    let mut down = Vec::new();
    let mut stay = Vec::new();
    let mut i = 0f32;
    while let Some(open_price) = open_iter.next() {
        let close_price = close_iter.next().unwrap();
        let volume = volume_iter.next().unwrap();
        if close_price > open_price {
            up.extend(&[
                Vertex {
                    position: [i - 0.4, *volume as f32],
                },
                Vertex {
                    position: [i + 0.4, *volume as f32],
                },
                Vertex {
                    position: [i + 0.4, *volume as f32],
                },
                Vertex {
                    position: [i + 0.4, 0.0],
                },
                Vertex {
                    position: [i + 0.4, 0.0],
                },
                Vertex {
                    position: [i - 0.4, 0.0],
                },
                Vertex {
                    position: [i - 0.4, 0.0],
                },
                Vertex {
                    position: [i - 0.4, *volume as f32],
                },
            ]);
        } else if close_price < open_price {
            down.extend(&[
                Vertex {
                    position: [i - 0.4, *volume as f32],
                },
                Vertex {
                    position: [i - 0.4, 0.0],
                },
                Vertex {
                    position: [i + 0.4, *volume as f32],
                },
                Vertex {
                    position: [i + 0.4, *volume as f32],
                },
                Vertex {
                    position: [i - 0.4, 0.0],
                },
                Vertex {
                    position: [i + 0.4, 0.0],
                },
            ]);
        } else {
            stay.extend(&[
                Vertex {
                    position: [i - 0.4, *volume as f32],
                },
                Vertex {
                    position: [i + 0.4, *volume as f32],
                },
                Vertex {
                    position: [i + 0.4, *volume as f32],
                },
                Vertex {
                    position: [i + 0.4, 0.0],
                },
                Vertex {
                    position: [i + 0.4, 0.0],
                },
                Vertex {
                    position: [i - 0.4, 0.0],
                },
                Vertex {
                    position: [i - 0.4, 0.0],
                },
                Vertex {
                    position: [i - 0.4, *volume as f32],
                },
            ]);
        }
        i += 1.0;
    }
    VolumeVertex { up, down, stay }
});

pub static TRADE_PAIRS_VERTEX: LazyLock<TradePairVertex> = LazyLock::new(|| {
    let mut profit = Vec::new();
    let mut loss = Vec::new();
    let mut buy = Vec::new();
    let mut sell = Vec::new();
    let mut short = Vec::new();
    let mut cover = Vec::new();
    let mut buy_text = Vec::new();
    let mut sell_text = Vec::new();
    let mut short_text = Vec::new();
    let mut cover_text = Vec::new();
    let mut start_ix = 0;
    for d in TRADE_PAIRS.iter() {
        let open_ix = search_ix_by_dt(d.open_dt, start_ix);
        assert!(open_ix >= 0);
        let close_ix = search_ix_by_dt(d.close_dt, start_ix);
        assert!(close_ix >= 0);
        start_ix = open_ix as usize;
        let open_price = d.open_price;
        let close_price = d.close_price;

        //交易对连线
        let is_profit: bool;
        if d.direction == LONG && close_price >= open_price {
            is_profit = true;
        } else if d.direction == SHORT && close_price <= open_price {
            is_profit = true;
        } else {
            is_profit = false;
        }

        if is_profit {
            profit.extend(&[
                Vertex {
                    position: [open_ix as f32, open_price as f32],
                },
                Vertex {
                    position: [close_ix as f32, close_price as f32],
                },
            ]);
        } else {
            loss.extend(&[
                Vertex {
                    position: [open_ix as f32, open_price as f32],
                },
                Vertex {
                    position: [close_ix as f32, close_price as f32],
                },
            ]);
        }

        //交易对三角
        if d.direction == LONG {
            let open_y = HISTORY.low_price[open_ix as usize];
            let close_y = HISTORY.high_price[close_ix as usize];
            buy.extend(
                &[Vertex {
                    position: [open_ix as f32, open_y as f32],
                }]
                .repeat(3),
            );
            sell.extend(
                &[Vertex {
                    position: [close_ix as f32, close_y as f32],
                }]
                .repeat(3),
            );
            buy_text.push((
                Vec4::from_array([open_ix as f32, open_y as f32, 0.0, 1.0]),
                format!("{}", d.volume),
            ));
            sell_text.push((
                Vec4::from_array([close_ix as f32, close_y as f32, 0.0, 1.0]),
                format!("{}", d.volume),
            ));
        } else {
            let open_y = HISTORY.high_price[open_ix as usize];
            let close_y = HISTORY.low_price[close_ix as usize];
            short.extend(
                &[Vertex {
                    position: [open_ix as f32, open_y as f32],
                }]
                .repeat(3),
            );
            cover.extend(
                &[Vertex {
                    position: [close_ix as f32, close_y as f32],
                }]
                .repeat(3),
            );
            short_text.push((
                Vec4::from_array([open_ix as f32, open_y as f32, 0.0, 1.0]),
                format!("{}", d.volume),
            ));
            cover_text.push((
                Vec4::from_array([close_ix as f32, close_y as f32, 0.0, 1.0]),
                format!("{}", d.volume),
            ));
        }
    }
    TradePairVertex {
        profit,
        loss,
        buy,
        sell,
        short,
        cover,
        buy_text,
        sell_text,
        short_text,
        cover_text,
    }
});

fn search_ix_by_dt(datetime: NaiveDateTime, mut start_ix: usize) -> i64 {
    while start_ix < HISTORY.datetime.len() {
        if HISTORY.datetime[start_ix] == datetime {
            return start_ix as i64;
        }
        start_ix += 1;
    }
    -1
}
