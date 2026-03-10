FactoryBot.define do
  factory :tagging do
    tag
    association :taggable, factory: :track
    user
  end
end
