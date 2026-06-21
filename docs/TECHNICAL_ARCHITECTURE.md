# ShelfBrain Technical Architecture

## Recommended MVP Stack

### Mobile App

- React Native
- Expo
- Expo Router
- TypeScript
- React Native Paper or Tamagui for UI primitives
- Zustand or React Context for local app state
- TanStack Query once backend sync is introduced

### Backend

- Supabase Postgres
- Supabase Auth for MVP authentication
- Supabase Row Level Security
- Supabase Realtime for shared shelf updates if needed
- Supabase Storage later for attachments

### Authentication

Start with Supabase Auth for the MVP unless there is a specific Auth0 requirement.

Auth0 can be introduced later if ShelfBrain needs:

- Enterprise identity
- Advanced social login configuration
- Complex account linking
- Organization-level auth
- A separate auth provider from the data platform

Using Auth0 from day one is possible, but it adds integration work before the product loop is validated.

### Rust

Rust is a good fit for background and performance-sensitive work, but it should not be required for the first mobile prototype.

Potential Rust uses after the MVP:

- Background reminder scheduler
- Local item indexing
- Offline-first sync engine
- Cross-platform notification rule engine
- AI preprocessing pipeline
- High-performance import/export tools

For the Expo MVP, prefer platform-supported notifications first:

- `expo-notifications`
- Supabase scheduled functions or edge functions for server-side reminder checks

Introduce Rust when there is a concrete bottleneck or native background requirement that JavaScript, Expo, or Supabase cannot satisfy cleanly.

## Architecture Phases

### Phase 1: Local Prototype

Goal: prove the interaction model.

Use:

- Expo app
- Local component state or lightweight store
- Hardcoded categories
- No auth
- No backend

Screens:

- Shelf home
- Quick capture
- Shelf detail
- Item detail/edit

### Phase 2: Persistent Personal App

Goal: make the app useful across sessions and devices.

Use:

- Supabase Auth
- Supabase Postgres
- TanStack Query
- Row Level Security
- Push notification setup

Features:

- User accounts
- Persisted shelves
- Persisted items
- Categories
- Due dates
- Archive

### Phase 3: Shared Shelves

Goal: support small-group attention spaces.

Use:

- Shelf members table
- Row Level Security policies
- Invite flow
- Optional Supabase Realtime

Features:

- Invite user by email
- Shared shelf membership
- Collaborative item updates
- Simple activity indicators

### Phase 4: Intelligent Resurfacing

Goal: make ShelfBrain feel alive without becoming noisy.

Use:

- Notification preferences
- Scheduled jobs
- Review prompts
- Later: AI categorization and shelf suggestions

Features:

- Daily Top Shelf digest
- Weekly Middle Shelf check-in
- Monthly or quarterly Bottom Shelf review
- Dormant item prompts

## Suggested Repository Structure

```text
shelf-brain/
  workspaces/
    mobile/
      app/
      src/
        components/
        features/
        lib/
        stores/
        types/
  supabase/
    migrations/
    seed.sql
  rust/
    shelfbrain-worker/
  docs/
```

The `rust/` directory should stay empty or experimental until the app has a confirmed background-work requirement.

## Supabase Tables

### `profiles`

- `id uuid primary key references auth.users`
- `email text`
- `display_name text`
- `created_at timestamptz`

### `shelves`

- `id uuid primary key`
- `owner_id uuid references profiles`
- `name text`
- `type text check (type in ('top', 'middle', 'bottom', 'custom'))`
- `is_shared boolean`
- `created_at timestamptz`
- `updated_at timestamptz`

### `categories`

- `id uuid primary key`
- `user_id uuid references profiles`
- `name text`
- `created_at timestamptz`

### `items`

- `id uuid primary key`
- `shelf_id uuid references shelves`
- `created_by uuid references profiles`
- `title text`
- `category_id uuid references categories`
- `due_at timestamptz`
- `notes text`
- `status text check (status in ('active', 'archived'))`
- `created_at timestamptz`
- `updated_at timestamptz`
- `last_reviewed_at timestamptz`

### `shelf_members`

- `id uuid primary key`
- `shelf_id uuid references shelves`
- `user_id uuid references profiles`
- `role text check (role in ('owner', 'member'))`
- `created_at timestamptz`

### `notification_preferences`

- `id uuid primary key`
- `user_id uuid references profiles`
- `shelf_type text check (shelf_type in ('top', 'middle', 'bottom'))`
- `frequency text`
- `enabled boolean`
- `created_at timestamptz`
- `updated_at timestamptz`

## Open Technical Questions

- Should personal shelves be fixed records created for each user, or virtual shelves derived from item shelf type?
- Should categories be global defaults, user-created, or both?
- Should shared shelves use the same Top/Middle/Bottom model inside each shared context?
- Should notifications be local-first or server-driven?
- Should Auth0 be added only when there is a non-MVP auth requirement?
