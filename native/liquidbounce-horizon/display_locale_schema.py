"""Write owned display translations into the schema consumed by the native UI."""
def put_module(data, prefix, name, entries, english):
    for key in list(data):
        if key == prefix or key.startswith(prefix + '.'):
            del data[key]
    data.setdefault('moduleNames', {})[name] = entries['']
    labels = data.setdefault('labels', {})
    descriptions = data.setdefault('descriptions', {})
    for suffix, text in entries.items():
        if suffix == 'description' or suffix.endswith('.description'):
            descriptions[prefix + '.' + suffix] = text
        elif suffix:
            # Configurable.name and tagged choices keep their original English
            # identifiers; only this display lookup changes their captions.
            tail = suffix.rsplit('.', 1)[-1]
            labels[tail[0].upper() + tail[1:]] = text
    if 'englishDescriptions' in data:
        data['englishDescriptions'].update({prefix + '.' + suffix: text
            for suffix, text in english.items()})
