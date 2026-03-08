FactoryBot.define do
  factory :api_key do
    name { "Test API Key" }
    secret_key { "test_secret_key_123" }
    user
  end
end
