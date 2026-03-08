class MessageResource < Madmin::Resource
  attribute :id, form: false
  attribute :chat
  attribute :role
  attribute :content, index: false
  attribute :content_raw, index: false, show: false
  attribute :model, index: false
  attribute :input_tokens, index: false
  attribute :output_tokens, index: false
  attribute :cached_tokens, index: false, show: false
  attribute :cache_creation_tokens, index: false, show: false
  attribute :thinking_text, index: false, show: false
  attribute :thinking_signature, index: false, show: false
  attribute :thinking_tokens, index: false, show: false
  attribute :tool_call_id, index: false
  attribute :attachments, index: false
  attribute :tool_calls, index: false
  attribute :parent_tool_call, index: false
  attribute :tool_results, index: false
  attribute :created_at, form: false
  attribute :updated_at, form: false, index: false

  def self.display_name(record) = "#{record.role} ##{record.id}"
  def self.default_sort_column = "created_at"
  def self.default_sort_direction = "desc"
end
