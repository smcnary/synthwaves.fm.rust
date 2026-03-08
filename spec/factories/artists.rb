FactoryBot.define do
  factory :artist do
    sequence(:name) { |n| "Artist #{n}" }
    category { "music" }

    trait :podcast do
      category { "podcast" }
    end
  end
end
