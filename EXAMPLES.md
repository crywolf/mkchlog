# Detailed explanation

Final changelog output can have "title" and "description", but only "title" is required. By defaut the first line in the commit message is used as "title" and the rest is used as "description" so the whole commit message will be included in the changelog output.

This is OK when the commit message is equally useful for developers and users.

Often this is not the case. If you want to use differrent title or description, use `title` and/or `description` field in the `changelog:` part of the commit message.

## Cookbook examples

```yaml
This is the simplest version

changelog:
  section: features
```

```yaml
This is "title" from the commit message

This is "description" from the commit message. It can be longer.

changelog:
  section: features
```

```yaml
This "title" will not be present in changelog output

This "description" will not be present in changelog output as well.

changelog:
  section: features
  title: This will override "title" from the commit message
  description: This will override "description" from the commit message
```

```yaml
This is "title" from the commit message

This "description" will not be present in changelog output.

changelog:
  section: features
  only-title: true
```

```yaml
This is "title" from the commit message

This "description" will not be present in changelog output.

changelog:
  section: features
  description: This will override "description" from the commit message
```

```yaml
This "title" will not be present in changelog output

This "description" will not be present in changelog output.

changelog:
  section: features
  title: This will override "title" from the commit message
  only-title: true
```
