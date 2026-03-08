# Groovy Tunes

A Rails application generated with [Boilercode](https://boilercode.io).

## Table of Contents

- [Getting Started](#getting-started)
- [Authentication](#authentication)
- [Admin Dashboard](#admin-dashboard)
- [API](#api)
- [Analytics](#analytics)
- [Utilities](#utilities)
- [Testing](#testing)

## Getting Started

### Requirements

- Ruby 4.0.1+
- SQLite3

### Setup

```bash
bin/setup
```

This installs dependencies, prepares the database, and starts the development server.

### Development

```bash
bin/dev
```

This starts the Rails server and Tailwind CSS watcher.

## Authentication

### User Authentication

This app uses Rails 8's built-in authentication system.

**Default Admin User:**
- Email: `admin@example.com`
- Password: `abc123`

**Creating New Users:**

```ruby
User.create(
  email_address: "user@example.com",
  password: "your-password",
  admin: false
)
```

**Admin Access:**

Admin users have access to protected admin routes. Set `admin: true` on a user to grant admin privileges:

```ruby
user.update(admin: true)
```

## Admin Dashboard

### Admin Panel (Madmin)

The admin panel is available at `/admin` for admin users.

**Features:**
- Auto-generated CRUD interfaces for all models
- Search and filtering capabilities
- Customizable dashboards

**Customizing Admin Resources:**

Admin resources are in `app/madmin/resources/`. To customize a resource:

```ruby
# app/madmin/resources/user_resource.rb
class UserResource < Madmin::Resource
  attribute :id, form: false
  attribute :email_address
  attribute :admin
  attribute :created_at, form: false
end
```

### Job Monitoring (Mission Control)

Monitor and manage background jobs at `/admin/jobs` (or `/jobs` if Madmin is not installed).

**Features:**
- View pending, running, and completed jobs
- Retry failed jobs
- Pause and resume queues
- Real-time job statistics

### Maintenance Tasks

Run and monitor maintenance tasks at `/admin/maintenance_tasks` (or `/maintenance_tasks` if Madmin is not installed).

**Creating a Task:**

```bash
bin/rails generate maintenance_tasks:task update_user_data
```

```ruby
# app/tasks/maintenance/update_user_data_task.rb
module Maintenance
  class UpdateUserDataTask < MaintenanceTasks::Task
    def collection
      User.all
    end

    def process(user)
      user.update!(processed_at: Time.current)
    end
  end
end
```

**Running Tasks:**

Tasks can be run from the web UI or via the command line:

```bash
bin/rails maintenance_tasks:run Maintenance::UpdateUserDataTask
```

### Feature Flags (Flipper)

Manage feature flags at `/admin/flipper` (or `/flipper` if Madmin is not installed).

**Usage in Code:**

```ruby
# Check if a feature is enabled
if Flipper.enabled?(:new_dashboard)
  # show new dashboard
end

# Enable for specific users
Flipper.enable(:beta_feature, current_user)

# Enable for a percentage of users
Flipper.enable_percentage_of_actors(:new_feature, 25)
```

**In Views:**

```erb
<% if Flipper.enabled?(:new_feature, current_user) %>
  <%= render "new_feature" %>
<% end %>
```

## API

### API Endpoints

This app includes a versioned JSON API with JWT authentication.

**Authentication Flow:**

1. Create an API key from the web UI at `/api_keys`
2. Request a JWT token:

```bash
curl -X POST http://localhost:3000/api/v1/auth/token \
  -H "Content-Type: application/json" \
  -d '{"client_id": "your_client_id", "secret_key": "your_secret_key"}'
```

3. Use the token in subsequent requests:

```bash
curl http://localhost:3000/api/v1/your_endpoint \
  -H "Authorization: Bearer your_jwt_token"
```

**Creating API Endpoints:**

Add new endpoints in `app/controllers/api/v1/`. Inherit from `Api::V1::BaseController` for automatic JWT authentication:

```ruby
module Api
  module V1
    class UsersController < BaseController
      def index
        render json: User.all
      end
    end
  end
end
```

**Managing API Keys:**

Users can manage their API keys at `/api_keys`. Each key has a client ID and secret key that can be used to obtain JWT tokens.

## Analytics

### Analytics (Ahoy)

Ahoy tracks visits and events in your application.

**Tracking Events:**

```ruby
# In controllers
ahoy.track "Viewed product", product_id: product.id

# In views
<%= ahoy.track "Viewed landing page" %>
```

**Querying Data:**

```ruby
# Recent visits
Ahoy::Visit.last(10)

# Events for a specific action
Ahoy::Event.where(name: "Viewed product")
```

**Configuration:**

Customize tracking in `config/initializers/ahoy.rb`.

## Utilities

### Pagination (Pagy)

Pagy is configured for efficient pagination.

**In Controllers:**

```ruby
def index
  @pagy, @users = pagy(User.all)
end
```

**In Views:**

```erb
<%= pagy_nav(@pagy) %>
```

**Customizing:**

```ruby
# Change items per page
@pagy, @users = pagy(User.all, limit: 25)
```

### Column Sorting

Sortable columns are available in index views.

**In Controllers:**

```ruby
def index
  @users = apply_order(User.all)
end
```

**In Views:**

```erb
<th><%= sort_link("Name", :name) %></th>
<th><%= sort_link("Created", :created_at) %></th>
```

**Allowed Columns:**

By default, sorting is allowed on any column. To restrict:

```ruby
def orderable_columns
  %w[name email created_at]
end
```

### File Uploads (Active Storage)

Active Storage is configured for file uploads.

**Adding Attachments to Models:**

```ruby
class User < ApplicationRecord
  has_one_attached :avatar
  has_many_attached :documents
end
```

**In Forms:**

```erb
<%= form.file_field :avatar %>
<%= form.file_field :documents, multiple: true %>
```

**Displaying Images:**

```erb
<%= image_tag user.avatar if user.avatar.attached? %>
```

**S3 Configuration (if enabled):**

Add your S3 credentials to `config/credentials.yml.enc`:

```yaml
amazon:
  access_key_id: YOUR_ACCESS_KEY
  secret_access_key: YOUR_SECRET_KEY
  region: us-east-1
  bucket: your-bucket-name
```

### AI Integration (RubyLLM)

RubyLLM provides a simple interface for AI-powered features.

**Configuration:**

Set your API key in credentials or environment:

```yaml
# config/credentials.yml.enc
openai:
  api_key: YOUR_API_KEY
```

Or set `OPENAI_API_KEY` in your environment.

**Basic Usage:**

```ruby
response = RubyLLM.chat("What is Ruby on Rails?")
puts response.content
```

**With Conversation History:**

```ruby
chat = RubyLLM.chat
chat.ask("What is Ruby?")
chat.ask("How does it compare to Python?")
```

**Streaming Responses:**

```ruby
RubyLLM.chat("Tell me a story") do |chunk|
  print chunk.content
end
```

## Testing

### Running Tests

```bash
# Run all tests
bin/rspec

# Run specific file
bin/rspec spec/models/user_spec.rb

# Run specific test by line number
bin/rspec spec/models/user_spec.rb:42
```

**Test Helpers:**

FactoryBot is available for creating test data:

```ruby
# Create a user
user = create(:user)

# Build without saving
user = build(:user, email_address: "custom@example.com")
```

Shoulda Matchers are available for concise model tests:

```ruby
RSpec.describe User, type: :model do
  it { should validate_presence_of(:email_address) }
  it { should have_many(:api_keys) }
end
```

---

Generated with [Boilercode](https://boilercode.io)
