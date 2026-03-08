class ModelResource < Madmin::Resource
  attribute :id, form: false
  attribute :model_id
  attribute :name
  attribute :provider
  attribute :family
  attribute :context_window
  attribute :max_output_tokens, index: false
  attribute :model_created_at, index: false
  attribute :knowledge_cutoff, index: false
  attribute :modalities, index: false
  attribute :capabilities, index: false
  attribute :pricing, index: false
  attribute :metadata, index: false
  attribute :chats, index: false
  attribute :created_at, form: false, index: false
  attribute :updated_at, form: false, index: false

  def self.display_name(record) = record.name
  def self.default_sort_column = "name"
end
