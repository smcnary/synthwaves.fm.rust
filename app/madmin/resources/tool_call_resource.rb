class ToolCallResource < Madmin::Resource
  attribute :id, form: false
  attribute :name
  attribute :tool_call_id
  attribute :message
  attribute :arguments, index: false
  attribute :thought_signature, index: false, show: false
  attribute :result, index: false
  attribute :created_at, form: false, index: false
  attribute :updated_at, form: false, index: false

  def self.display_name(record) = record.name
  def self.default_sort_column = "created_at"
  def self.default_sort_direction = "desc"
end
