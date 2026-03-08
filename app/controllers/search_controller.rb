class SearchController < ApplicationController
  def index
    @query = params[:q].to_s.strip
    if @query.present?
      @results = SearchService.call(query: @query)
    else
      @results = {artists: [], albums: [], tracks: []}
    end
  end

  def dropdown
    @query = params[:q].to_s.strip
    if @query.present?
      @results = SearchService.call(query: @query, limit: 5)
    else
      @results = {artists: [], albums: [], tracks: []}
    end
    render layout: false
  end
end
