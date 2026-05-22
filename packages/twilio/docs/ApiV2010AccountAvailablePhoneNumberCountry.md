# ApiV2010AccountAvailablePhoneNumberCountry

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**country_code** | Option<**String**> | The [ISO-3166-1](https://en.wikipedia.org/wiki/ISO_3166-1_alpha-2) country code of the country. | [optional]
**country** | Option<**String**> | The name of the country. | [optional]
**uri** | Option<**String**> | The URI of the Country resource, relative to `https://api.twilio.com`. | [optional]
**beta** | Option<**bool**> | Whether all phone numbers available in the country are new to the Twilio platform. `true` if they are and `false` if all numbers are not in the Twilio Phone Number Beta program. | [optional]
**subresource_uris** | Option<**serde_json::Value**> | A list of related AvailablePhoneNumber resources identified by their URIs relative to `https://api.twilio.com`. | [optional]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


