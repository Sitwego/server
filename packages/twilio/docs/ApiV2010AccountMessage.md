# ApiV2010AccountMessage

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**body** | Option<**String**> | The text content of the message | [optional]
**num_segments** | Option<**String**> | The number of segments that make up the complete message. SMS message bodies that exceed the [character limit](https://www.twilio.com/docs/glossary/what-sms-character-limit) are segmented and charged as multiple messages. Note: For messages sent via a Messaging Service, `num_segments` is initially `0`, since a sender hasn't yet been assigned. | [optional]
**direction** | Option<[**models::MessageEnumDirection**](MessageEnumDirection.md)> |  | [optional]
**from** | Option<**String**> | The sender's phone number (in [E.164](https://en.wikipedia.org/wiki/E.164) format), [alphanumeric sender ID](https://www.twilio.com/docs/sms/quickstart), [Wireless SIM](https://www.twilio.com/docs/iot/wireless/programmable-wireless-send-machine-machine-sms-commands), [short code](https://www.twilio.com/en-us/messaging/channels/sms/short-codes), or  [channel address](https://www.twilio.com/docs/messaging/channels) (e.g., `whatsapp:+15554449999`). For incoming messages, this is the number or channel address of the sender. For outgoing messages, this value is a Twilio phone number, alphanumeric sender ID, short code, or channel address from which the message is sent. | [optional]
**to** | Option<**String**> | The recipient's phone number (in [E.164](https://en.wikipedia.org/wiki/E.164) format) or [channel address](https://www.twilio.com/docs/messaging/channels) (e.g. `whatsapp:+15552229999`) | [optional]
**date_updated** | Option<**String**> | The [RFC 2822](https://datatracker.ietf.org/doc/html/rfc2822#section-3.3) timestamp (in GMT) of when the Message resource was last updated | [optional]
**price** | Option<**String**> | The amount billed for the message in the currency specified by `price_unit`. The `price` is populated after the message has been sent/received, and may not be immediately availalble. View the [Pricing page](https://www.twilio.com/en-us/pricing) for more details. | [optional]
**error_message** | Option<**String**> | The description of the `error_code` if the Message `status` is `failed` or `undelivered`. If no error was encountered, the value is `null`. The value returned in this field for a specific error cause is subject to change as Twilio improves errors. Users should not use the `error_code` and `error_message` fields programmatically. | [optional]
**uri** | Option<**String**> | The URI of the Message resource, relative to `https://api.twilio.com`. | [optional]
**account_sid** | Option<**String**> | The SID of the [Account](https://www.twilio.com/docs/iam/api/account) associated with the Message resource | [optional]
**num_media** | Option<**String**> | The number of media files associated with the Message resource. | [optional]
**status** | Option<[**models::MessageEnumStatus**](MessageEnumStatus.md)> |  | [optional]
**messaging_service_sid** | Option<**String**> | The SID of the [Messaging Service](https://www.twilio.com/docs/messaging/api/service-resource) associated with the Message resource. A unique default value is assigned if a Messaging Service is not used. | [optional]
**sid** | Option<**String**> | The unique, Twilio-provided string that identifies the Message resource. | [optional]
**date_sent** | Option<**String**> | The [RFC 2822](https://datatracker.ietf.org/doc/html/rfc2822#section-3.3) timestamp (in GMT) of when the Message was sent. For an outgoing message, this is when Twilio sent the message. For an incoming message, this is when Twilio sent the HTTP request to your incoming message webhook URL. | [optional]
**date_created** | Option<**String**> | The [RFC 2822](https://datatracker.ietf.org/doc/html/rfc2822#section-3.3) timestamp (in GMT) of when the Message resource was created | [optional]
**error_code** | Option<**i32**> | The [error code](https://www.twilio.com/docs/api/errors) returned if the Message `status` is `failed` or `undelivered`. If no error was encountered, the value is `null`. The value returned in this field for a specific error cause is subject to change as Twilio improves errors. Users should not use the `error_code` and `error_message` fields programmatically. | [optional]
**price_unit** | Option<**String**> | The currency in which `price` is measured, in [ISO 4127](https://www.iso.org/iso/home/standards/currency_codes.htm) format (e.g. `usd`, `eur`, `jpy`). | [optional]
**api_version** | Option<**String**> | The API version used to process the Message | [optional]
**subresource_uris** | Option<**serde_json::Value**> | A list of related resources identified by their URIs relative to `https://api.twilio.com` | [optional]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


