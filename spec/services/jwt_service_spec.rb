require "rails_helper"

RSpec.describe JWTService do
  describe ".encode" do
    it "encodes a payload into a JWT token" do
      payload = {user_id: 1, api_key_id: 2}
      token = described_class.encode(payload)

      expect(token).to be_a(String)
      expect(token.split(".").length).to eq(3)
    end

    it "includes expiration in the payload" do
      payload = {user_id: 1}
      token = described_class.encode(payload, exp: 2.hours.from_now)

      decoded = JWT.decode(token, Rails.application.credentials.secret_key_base, true, algorithm: "HS256").first
      expect(decoded["exp"]).to be_present
    end

    it "uses custom expiration when provided" do
      custom_exp = 30.minutes.from_now
      token = described_class.encode({user_id: 1}, exp: custom_exp)

      decoded = JWT.decode(token, Rails.application.credentials.secret_key_base, true, algorithm: "HS256").first
      expect(decoded["exp"]).to eq(custom_exp.to_i)
    end
  end

  describe ".decode" do
    it "decodes a valid token" do
      payload = {user_id: 1, api_key_id: 2}
      token = described_class.encode(payload)

      decoded = described_class.decode(token)

      expect(decoded[:user_id]).to eq(1)
      expect(decoded[:api_key_id]).to eq(2)
    end

    it "returns nil for an invalid token" do
      decoded = described_class.decode("invalid.token.here")
      expect(decoded).to be_nil
    end

    it "returns nil for an expired token" do
      token = described_class.encode({user_id: 1}, exp: 1.second.ago)
      decoded = described_class.decode(token)
      expect(decoded).to be_nil
    end

    it "returns nil for a tampered token" do
      token = described_class.encode({user_id: 1})
      tampered_token = token[0..-5] + "xxxx"

      decoded = described_class.decode(tampered_token)
      expect(decoded).to be_nil
    end

    it "returns hash with indifferent access" do
      token = described_class.encode({user_id: 1})
      decoded = described_class.decode(token)

      expect(decoded["user_id"]).to eq(1)
      expect(decoded[:user_id]).to eq(1)
    end
  end
end
