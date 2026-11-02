## ADDED Requirements

### Requirement: Sortable compact session list
The resume picker SHALL render each session as a compact row that fits the list width and SHALL let the user change sort order.

#### Scenario: Cycle sort
- **WHEN** the user presses `o`
- **THEN** the order SHALL cycle recency, title, saved-first, creation, and the title bar SHALL show the active mode

#### Scenario: Search
- **WHEN** the user enters a search query
- **THEN** results SHALL sort by relevance and the search bar SHALL show matched/total counts

#### Scenario: Narrow terminal
- **WHEN** the terminal is narrower than 80 columns
- **THEN** no row SHALL wrap and text SHALL truncate by display cells
