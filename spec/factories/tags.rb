FactoryBot.define do
  factory :tag do
    sequence(:name) { |n| "Tag #{n}" }
    tag_type { "genre" }

    trait :mood do
      tag_type { "mood" }
    end
  end
end
