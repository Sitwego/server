# Sit-We-Go

**A community-driven, open-source mobility platform**

Sit-We-Go is an open mobility platform built to empower drivers and riders alike. This repository hosts the backend services that power the Sit-We-Go ride-hailing experience.

## Vision
Sit-We-Go aims to empower service providers with a high-tech, cost-effective platform built on open foundations. Our vision is centered on the following principles:

1. **Driver-Centric**: Ensuring fair earnings and economic empowerment for drivers, with minimal commissions and transparent pricing.
2. **Open**: Utilizing open code, open data, and open standards to foster transparency, innovation, and collaboration within the community.
3. **Optimize**: Achieving sustainable, population-scale growth through continuous optimization of infrastructure, mapping, and development costs while maintaining high reliability and a rich user experience.
4. **Multimodal**: Supporting various modes of transport to provide comprehensive, integrated mobility solutions for a seamless user experience.
5. **Shared Transport**: Promoting shared mobility options to reduce traffic congestion, lower costs, and minimize carbon emissions, contributing to a sustainable future.

As engineers, we often seek opportunities where one person can serve millions of others. Sit-We-Go is an effort to enable the careers of those who serve one-to-one — drivers who transport people each and every day. These drivers indirectly serve millions of customers over time.

## Core Values

### Community-first
Mobility must be owned by the community of citizens and drivers who collaborate to create a thriving, equitable, and sustainable environment. Participatory development from the community helps build products to solve large-scale problems like mobility.

### Citizens as owners
Drivers invest in their vehicles and work hard every day. Citizens pay for and use the service. They both should own the platform, offering quality service at a fair price, without intermediaries dictating terms or prices.

### Open Platform
Customers participate in community efforts by using the open system, providing feedback, and improving drivers' lives. Our data and roadmap are open for feedback. The team strives to be efficient, prioritize the critical few, and do more with less.

### Tech and People are enablers
Mobility is an engineering problem, and technology can make it more efficient, reliable, and sustainable. We need both tech innovation and human involvement. Empathy and support for both citizens and drivers are crucial.

### Sustainable Growth
We aim to solve complex, long-term problems sustainably, avoiding unsustainable tactics like discounts and incentives. We pursue initiatives that are financially, environmentally, and socially sustainable, promoting shared mobility and efficient public transportation to reduce traffic, cost, and carbon emissions.

## Why Solve This Problem?
Mobility is critical to economic growth, social progress, and individual well-being. People's livelihoods depend on mobility, but current systems could be more efficient, sustainable, and accessible to the masses. To improve this, mobility should be community-driven, open, tech-enabled, and shared.

Join us in this endeavor to transform mobility. We look forward to your contribution and support!

## Workspace Layout

This repository is a Rust Cargo workspace. Each crate lives under [packages/](packages/):

| Package | Description |
|---|---|
| [packages/api](packages/api) | Main HTTP service binary (drivers, customers, rides, ride requests, ratings, subscriptions, etc.) |
| [packages/aws](packages/aws) | AWS integrations (S3, KMS) |
| [packages/db_store](packages/db_store) | PostgreSQL access layer |
| [packages/redis_store](packages/redis_store) | Redis access layer and event streams |
| [packages/kafka](packages/kafka) | Kafka producer/consumer wiring |
| [packages/payment](packages/payment) | Payment providers (M-Pesa) |
| [packages/notif_api](packages/notif_api) | Notification API client |
| [packages/email_api](packages/email_api) | Email API client |
| [packages/sms_api](packages/sms_api) | SMS API client |
| [packages/twilio](packages/twilio) | Twilio integration |
| [packages/utils](packages/utils) | Shared utilities (hashing, HTTP helpers) |
| [packages/shared_macro](packages/shared_macro) | Shared procedural macros |

## Get Involved
Explore the code, provide feedback, and contribute to the project. Together, we can create a scalable, efficient, safe, and sustainable transportation network.

### Community Engagement
Join our [GitHub Discussions](https://github.com/Sitwego/backend/discussions) to ask questions and explore ideas.

### Contributing Guidelines
We believe in the power of open-source. This repository aims to foster an open development platform for the mobility stack, where anyone can inspect and contribute. We welcome contributions in the form of bug reports, code patches, documentation updates, feature requests, and notifications of breaking changes in dependencies.

Happy Contributing!

## License

AGPL-3.0-or-later
