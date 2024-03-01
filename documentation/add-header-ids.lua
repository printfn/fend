function generateSlug(el)
	if el.tag == "Header" then
		local headerText = pandoc.utils.stringify(el.content)
		local slug = urlify(headerText)
		el.attr = { id = slug }
	end
	return el
end

function urlify(text)
	-- Replace non-alphanumeric characters with hyphens
	text = text:gsub("[^a-zA-Z0-9]", "-")
	-- Remove extra hyphens
	text = text:gsub("-+", "-")
	-- Remove leading and trailing hyphens
	text = text:gsub("^%-*(.-)%-*$", "%1")
	-- Convert to lowercase
	text = text:lower()
	return text
end

-- Apply the filter
return {
	{ Header = generateSlug }
}
