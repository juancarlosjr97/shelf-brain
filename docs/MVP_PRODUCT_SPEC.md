# ShelfBrain MVP Product Spec

## Product Promise

ShelfBrain helps people capture thoughts quickly and decide how much attention each thing deserves.

The MVP should prove one behavior:

> A user can capture something in under 10 seconds, place it on a shelf, and trust that it will resurface at the right level of pressure.

## Target User

ShelfBrain is for people who collect lots of thoughts, errands, plans, ideas, recommendations, and long-term interests, but do not want to maintain a full productivity system.

They are not trying to manage every task perfectly. They want a low-friction place to put things so they can stop carrying them mentally.

## MVP Scope

### Must Have

- Create an item
- Assign item to Top, Middle, or Bottom Shelf
- Optional category
- Optional due date
- Optional notes
- View all shelves
- Move item between shelves
- Edit item
- Archive item
- Basic local or push notifications
- Shared shelves, if feasible without slowing the core app

### Should Have

- Fast capture from the home screen
- Category filter
- Due-soon indicators for Top Shelf items
- Simple review prompts for Middle and Bottom Shelf items
- Empty states that encourage capture, not setup

### Not MVP

- AI priority suggestions
- AI categorization
- AI shelf reviews
- Analytics and insights
- Unlimited attachments
- Complex workspaces
- Rich project management
- Full Auth0 integration
- Native Rust background engine unless a clear MVP limitation requires it

## Core Screens

### 1. Shelf Home

The main screen shows the three shelves as the primary navigation.

Each shelf should communicate attention level at a glance:

- Top Shelf: urgent, near-term, active
- Middle Shelf: steady, important, revisit soon
- Bottom Shelf: calm, future, low pressure

Suggested content per shelf row:

- Shelf name
- Short attention label
- Item count
- A few recent or due items

### 2. Quick Capture

The capture flow should be reachable from every main screen.

Required fields:

- Title
- Shelf

Optional fields:

- Category
- Due date
- Notes

Default behavior:

- If the user enters only a title, save it quickly.
- The default shelf can be Middle Shelf unless the user selects otherwise.
- Do not require category or due date.

### 3. Shelf Detail

Shows items within a shelf.

MVP interactions:

- Add item
- Edit item
- Move item to another shelf
- Archive item
- Filter by category
- Sort by due date or recently added

### 4. Item Detail

Shows the full item.

Fields:

- Title
- Shelf
- Category
- Due date
- Notes
- Created date
- Last reviewed date

Actions:

- Move shelf
- Edit
- Archive

### 5. Shared Shelf Detail

Only include in MVP if the core data model is already stable.

Fields:

- Shelf name
- Members
- Items
- Item activity

MVP sharing should be simple:

- Invite by email
- Role: owner or member
- Members can create, edit, move, and archive shared items

## Notification Model

Notifications should reinforce the attention model.

### Top Shelf

Purpose: keep near-term items visible.

Examples:

- Daily reminder digest
- Due date reminder
- Missed due date alert

### Middle Shelf

Purpose: prevent important things from disappearing.

Examples:

- Weekly check-in
- Monthly review prompt

### Bottom Shelf

Purpose: resurface ideas without pressure.

Examples:

- Monthly or quarterly review
- Long-dormant item prompt

Example copy:

> You added "Restaurant Inventory System" 12 months ago. Still interested?

Actions:

- Move to Top Shelf
- Move to Middle Shelf
- Keep on Bottom Shelf
- Archive

## Data Model

### User

- `id`
- `email`
- `display_name`
- `created_at`

### Shelf

- `id`
- `owner_id`
- `name`
- `type`: `top`, `middle`, `bottom`, or `custom`
- `is_shared`
- `created_at`
- `updated_at`

Personal shelves can be represented either as system shelves or as ordinary shelves with fixed `type` values.

### Item

- `id`
- `shelf_id`
- `created_by`
- `title`
- `category_id`
- `due_at`
- `notes`
- `status`: `active` or `archived`
- `created_at`
- `updated_at`
- `last_reviewed_at`

### Category

- `id`
- `user_id`
- `name`
- `created_at`

### Shelf Member

- `id`
- `shelf_id`
- `user_id`
- `role`: `owner` or `member`
- `created_at`

### Notification Preference

- `id`
- `user_id`
- `shelf_type`
- `frequency`
- `enabled`
- `created_at`
- `updated_at`

## Success Criteria

The MVP is working if:

- A new user understands the three shelves in under one minute.
- A user can capture an item in under 10 seconds.
- The app feels useful with fewer than 10 items.
- The app does not require projects, folders, or setup.
- Moving an item between shelves feels natural and central.

## First Build Milestone

Build a local-only Expo prototype first:

1. Shelf home
2. Quick capture
3. Shelf detail
4. Item edit
5. Archive

Then add persistence and sync:

1. Supabase schema
2. Auth
3. Row-level security
4. Shared shelves
5. Notifications
