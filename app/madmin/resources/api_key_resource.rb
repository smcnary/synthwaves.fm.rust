class APIKeyResource < Madmin::Resource
  attribute :id, form: false
  attribute :name
  attribute :user
  attribute :client_id
  attribute :expires_at
  attribute :last_used_at, index: false
  attribute :last_used_ip, index: false
  attribute :secret_key, index: false, show: false
  attribute :secret_key_confirmation, index: false, show: false
  attribute :created_at, form: false, index: false
  attribute :updated_at, form: false, index: false

  def self.display_name(record) = record.name
  def self.default_sort_column = "name"
end
