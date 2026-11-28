# fiscal spec

A retail 4-4-5 calendar, as used by the merchandising reports.

1. Fiscal year `FY` ends on the Saturday **nearest** January 31 of calendar
   year `FY + 1` (it can fall in early February). It starts the day after the
   previous fiscal year ends. `fiscal_year_end(fy)`, `fiscal_year_start(fy)`.
2. A fiscal year has 52 weeks, or 53 when its span is 371 days.
   `weeks_in_year(fy)`. Weeks run Sunday to Saturday.
3. `fiscal_week(d) -> (fy, week)`: week 1 starts on the fiscal year's first day.
4. Each quarter is 13 weeks split into periods of 4, 4 and 5 weeks, giving 12
   periods. In a 53-week year the extra week belongs to period 12.
   `fiscal_period(d) -> (fy, period)`.
5. `period_bounds(fy, period) -> (first_day, last_day)`, inclusive. A period
   outside 1-12 raises `ValueError`.
