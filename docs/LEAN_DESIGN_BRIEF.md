# ShelfBrain Lean Design Brief

## Design Goal

ShelfBrain should feel calm, immediate, and light.

The interface should help the user get a thought out of their head quickly, then gently manage how often it returns to their attention.

## Design Principles

### 1. Capture First

The capture action is the center of the product.

The user should never have to decide:

- Which project does this belong to?
- Which folder should I create?
- Which workflow should this enter?

They should only decide:

> Top, Middle, or Bottom?

### 2. Three Levels, No Clutter

The three shelves are the main information architecture.

Avoid:

- Nested folders
- Boards
- Project hierarchies
- Complex settings
- Dense dashboards

### 3. Mobile First

Design for one-handed use.

Important actions:

- Capture
- Move shelf
- Archive
- Set due date
- Add category

These should be reachable without digging through menus.

### 4. Calm Pressure

The shelves should feel emotionally different.

- Top Shelf: clear, active, urgent enough
- Middle Shelf: steady, warm, not forgotten
- Bottom Shelf: spacious, low pressure

Use visual priority without making the product feel stressful.

## First Screens To Pencil

### Shelf Home

Purpose: show attention distribution.

Elements:

- App title
- Three shelf sections
- Item counts
- Most relevant item previews
- Capture button

Key question:

> Can the user understand the app just from this screen?

### Quick Capture

Purpose: add something in under 10 seconds.

Elements:

- Title input
- Shelf selector
- Save button
- Optional expand area for category, due date, and notes

Key question:

> Can the user save without filling anything except title?

### Shelf Detail

Purpose: review one attention level.

Elements:

- Shelf name
- Category filter
- Sort control
- Item list
- Capture button scoped to this shelf

Key question:

> Does moving an item up or down feel obvious?

### Item Detail

Purpose: see and adjust one captured thought.

Elements:

- Title
- Shelf selector
- Category
- Due date
- Notes
- Archive action

Key question:

> Does this feel lighter than a task manager?

### Review Prompt

Purpose: resurface an item with minimal pressure.

Elements:

- Item title
- Time since added or reviewed
- Move to Top
- Move to Middle
- Keep
- Archive

Key question:

> Does resurfacing feel helpful rather than nagging?

## Visual Direction

Use a restrained, clean interface.

Suggested palette:

- Background: soft off-white or very light gray
- Text: near black
- Top Shelf accent: clear red or coral
- Middle Shelf accent: blue or green
- Bottom Shelf accent: muted violet or gray

Avoid making the whole app one dominant color.

## Interaction Notes

- Use segmented controls for shelf selection.
- Use icon buttons for archive, edit, and share.
- Use toggles for notification preferences.
- Use date picker controls for due dates.
- Use small category chips only when categories are present.

## MVP Copy

Use plain, low-pressure language.

Examples:

- "What needs your attention?"
- "Save to shelf"
- "Move up"
- "Move down"
- "Review later"
- "Still interested?"

Avoid productivity-heavy language like:

- "Workflow"
- "Sprint"
- "Execution"
- "Project pipeline"
- "Task dependency"

## Pencil Design Checklist

Before building UI, sketch:

- Home with empty shelves
- Home with 3 to 8 items
- Quick capture collapsed
- Quick capture expanded
- Top Shelf detail
- Bottom Shelf review prompt
- Shared shelf view

The sketches should answer layout and flow questions, not visual polish questions.
