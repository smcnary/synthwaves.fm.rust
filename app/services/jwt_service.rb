class JWTService
  SECRET_KEY = Rails.application.credentials.secret_key_base
  ALGORITHM = "HS256"

  def self.encode(payload, exp: 1.hour.from_now)
    payload[:exp] = exp.to_i
    JWT.encode(payload, SECRET_KEY, ALGORITHM)
  end

  def self.decode(token)
    JWT.decode(token, SECRET_KEY, true, algorithm: ALGORITHM).first.with_indifferent_access
  rescue JWT::DecodeError
    nil
  end
end
