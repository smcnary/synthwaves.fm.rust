class Tagging < ApplicationRecord
  belongs_to :tag
  belongs_to :taggable, polymorphic: true
  belongs_to :user

  validates :tag_id, uniqueness: {scope: [:taggable_type, :taggable_id, :user_id]}
end
