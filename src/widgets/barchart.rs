use crate::{
    buffer::Buffer,
    layout::Rect,
    style::Style,
    symbols,
    widgets::{Block, Widget},
};
use std::cmp::min;
use unicode_width::UnicodeWidthStr;

/// Display multiple bars in a single widgets
///
/// # Examples
///
/// ```
/// # use tui::widgets::{Block, Borders, BarChart};
/// # use tui::style::{Style, Color, Modifier};
/// BarChart::default()
///     .block(Block::default().title("BarChart").borders(Borders::ALL))
///     .bar_width(3)
///     .bar_gap(1)
///     .bar_style(Style::default().fg(Color::Yellow).bg(Color::Red))
///     .value_style(Style::default().fg(Color::Red).add_modifier(Modifier::BOLD))
///     .label_style(Style::default().fg(Color::White))
///     .data(&[("B0", 0), ("B1", 2), ("B2", 4), ("B3", 3)])
///     .max(4);
/// ```
#[derive(Debug, Clone)]
pub struct BarChart<'a> {
    /// Block to wrap the widget in
    pub block: Option<Block<'a>>,
    /// The width of each bar
    pub bar_width: u16,
    /// The gap between each bar
    pub bar_gap: u16,
    /// Set of symbols used to display the data
    pub bar_set: symbols::bar::Set,
    /// Style of the bars
    pub bar_style: Style,
    /// Style of the values printed at the bottom of each bar
    pub value_style: Style,
    /// Style of the labels printed under each bar
    pub label_style: Style,
    /// Style for the widget
    pub style: Style,
    /// Slice of (label, value) pair to plot on the chart.
    /// Cannot be modified directly, only with `set_data()`.
    data: &'a [(&'a str, u64)],
    /// Value necessary for a bar to reach the maximum height (if no value is specified,
    /// the maximum value in the data is taken as reference)
    pub max: Option<u64>,
    /// Values to display on the bar (computed when the data is passed to the widget)
    pub values: Vec<String>,
}

impl<'a> Default for BarChart<'a> {
    fn default() -> Self {
        Self {
            block: None,
            max: None,
            data: &[],
            values: Vec::new(),
            bar_style: Style::default(),
            bar_width: 1,
            bar_gap: 1,
            bar_set: symbols::bar::NINE_LEVELS,
            value_style: Default::default(),
            label_style: Default::default(),
            style: Default::default(),
        }
    }
}

impl<'a> BarChart<'a> {
    pub fn data(mut self, data: &'a [(&'a str, u64)]) -> Self {
        self.set_data(data);
        self
    }

    pub fn set_data(&mut self, data: &'a [(&'a str, u64)]) {
        self.data = data;
        self.values = Vec::with_capacity(self.data.len());
        for &(_, v) in self.data {
            self.values.push(format!("{}", v));
        }
    }

    pub fn block(mut self, block: Block<'a>) -> Self {
        self.block = Some(block);
        self
    }

    pub fn max(mut self, max: u64) -> Self {
        self.max = Some(max);
        self
    }

    pub fn bar_style(mut self, style: Style) -> Self {
        self.bar_style = style;
        self
    }

    pub fn bar_width(mut self, width: u16) -> Self {
        self.bar_width = width;
        self
    }

    pub fn bar_gap(mut self, gap: u16) -> Self {
        self.bar_gap = gap;
        self
    }

    pub fn bar_set(mut self, bar_set: symbols::bar::Set) -> Self {
        self.bar_set = bar_set;
        self
    }

    pub fn value_style(mut self, style: Style) -> Self {
        self.value_style = style;
        self
    }

    pub fn label_style(mut self, style: Style) -> Self {
        self.label_style = style;
        self
    }

    pub fn style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }
}

impl<'a> Widget for BarChart<'a> {
    fn render(&mut self, area: Rect, buf: &mut Buffer) {
        buf.set_style(area, self.style);

        let chart_area = match self.block.take() {
            Some(mut b) => {
                let inner_area = b.inner(area);
                b.render(area, buf);
                inner_area
            }
            None => area,
        };

        if chart_area.height < 2 {
            return;
        }

        let max = self
            .max
            .unwrap_or_else(|| self.data.iter().map(|t| t.1).max().unwrap_or_default());
        let max_index = min(
            (chart_area.width / (self.bar_width + self.bar_gap)) as usize,
            self.data.len(),
        );
        let mut data = self
            .data
            .iter()
            .take(max_index)
            .map(|&(l, v)| {
                (
                    l,
                    v * u64::from(chart_area.height - 1) * 8 / std::cmp::max(max, 1),
                )
            })
            .collect::<Vec<(&str, u64)>>();
        for j in (0..chart_area.height - 1).rev() {
            for (i, d) in data.iter_mut().enumerate() {
                let symbol = self.bar_set.symbol(d.1 as usize);

                for x in 0..self.bar_width {
                    buf.get_mut(
                        chart_area.left() + i as u16 * (self.bar_width + self.bar_gap) + x,
                        chart_area.top() + j,
                    )
                    .set_symbol(symbol)
                    .set_style(self.bar_style);
                }

                if d.1 > 8 {
                    d.1 -= 8;
                } else {
                    d.1 = 0;
                }
            }
        }

        for (i, &(label, value)) in self.data.iter().take(max_index).enumerate() {
            if value != 0 {
                let value_label = &self.values[i];
                let width = value_label.width() as u16;
                if width < self.bar_width {
                    buf.set_string(
                        chart_area.left()
                            + i as u16 * (self.bar_width + self.bar_gap)
                            + (self.bar_width - width) / 2,
                        chart_area.bottom() - 2,
                        value_label,
                        self.value_style,
                    );
                }
            }
            buf.set_stringn(
                chart_area.left() + i as u16 * (self.bar_width + self.bar_gap),
                chart_area.bottom() - 1,
                label,
                self.bar_width as usize,
                self.label_style,
            );
        }
    }
}
