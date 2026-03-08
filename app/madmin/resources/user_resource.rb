class UserResource < Madmin::Resource
  attribute :id, form: false
  attribute :name
  attribute :email_address
  attribute :admin
  attribute :password, index: false, show: false
  attribute :password_confirmation, index: false, show: false
  attribute :api_keys, index: false
  attribute :sessions, index: false
  attribute :created_at, form: false
  attribute :updated_at, form: false, index: false

  def self.display_name(record) = record.name
  def self.default_sort_column = "name"
end
