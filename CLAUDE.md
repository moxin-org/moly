# Important context
- Moly is an LLM app that allows users to interact with multiple AI providers, both local and remote
- MolyKit is an UI kit of reusable AI components, mainly a Chat widget, that allows adding an AI-assited chat to any Makepad application
- It's most important that the features we add to MolyKit, specially the protocol changes, are generic and reusable so that they can
be leveraged by multiple providers and clients.

# Implementation Requriemtents 
- All features must compile for all platforms, including desktop, mobile and web. 
Certain features are not yet supported in web therefore locked behind cfg flags.

# Code style
- Avoid unnecessary or obvious comments.
- Favor simple and elegant solutions over over-engineered ones.