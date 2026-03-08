class ChatResource < Madmin::Resource
  attribute :id, form: false
  attribute :model
  attribute :messages, index: false
  attribute :created_at, form: false
  attribute :updated_at, form: false, index: false

  def self.display_name(record) = "Chat ##{record.id}"
  def self.default_sort_column = "created_at"
  def self.default_sort_direction = "desc"
end
