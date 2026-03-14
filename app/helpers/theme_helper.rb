module ThemeHelper
  def current_theme
    theme = Current.user&.theme || Themeable::DEFAULT_THEME
    Themeable::THEMES.key?(theme) ? theme : Themeable::DEFAULT_THEME
  end

  def current_theme_config
    Themeable::THEMES[current_theme]
  end

  def current_theme_font_url
    current_theme_config[:font_url]
  end

  def current_theme_meta_color
    current_theme_config[:meta_color]
  end

  def theme_fonts_json
    Themeable::THEMES.transform_values { |v| v[:font_url] }.to_json
  end
end
