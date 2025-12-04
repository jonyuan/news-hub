# General Notes

## Architecture

We currently follow a "component-lite" [architecture](https://ratatui.rs/concepts/application-patterns/component-architecture/).

# Issues

- Some RSS feeds are being dropped due to unparsable fields.

# Todo List

## 12.3.25

- dynamic status message history component sizing
- scrollable features and scroll toggle between normal/message history
- move the status bar logic from the app into its handle_event() method
- fix unable to quit in detail_pane focus

## 12.1.25

- (low prio) group rss feeds, like Investing.com, in a more cohesive way, and migrate from hardcoded feeds
- handle any dropped rss items (e.g. unparsable date) ✅

## 11.30.25

- singleton search not loading details pane
- print to console interference ✅
- Long RSS GUID (permalink) overflow ✅

## 11.29.25

- deduplicate feed ✅
- search bar focus should not override feed commands ✅

## 11.26.25

- restructure to component-like ✅
- scrolling UI feature
- read pane ✅
- configure a more useful status bar ✅

## 11.24.25

- make the app work "out of the box" without API configurations ✅
