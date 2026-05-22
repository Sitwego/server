# ApiV2010AccountAvailablePhoneNumberCountryAvailablePhoneNumberNational

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**friendly_name** | Option<**String**> | A formatted version of the phone number. | [optional]
**phone_number** | Option<**String**> | The phone number in [E.164](https://www.twilio.com/docs/glossary/what-e164) format, which consists of a + followed by the country code and subscriber number. | [optional]
**lata** | Option<**String**> | The [LATA](https://en.wikipedia.org/wiki/Local_access_and_transport_area) of this phone number. Available for only phone numbers from the US and Canada. | [optional]
**locality** | Option<**String**> | The locality or city of this phone number's location. | [optional]
**rate_center** | Option<**String**> | The [rate center](https://en.wikipedia.org/wiki/Telephone_exchange) of this phone number. Available for only phone numbers from the US and Canada. | [optional]
**latitude** | Option<**f64**> | The latitude of this phone number's location. Available for only phone numbers from the US and Canada. | [optional]
**longitude** | Option<**f64**> | The longitude of this phone number's location. Available for only phone numbers from the US and Canada. | [optional]
**region** | Option<**String**> | The two-letter state or province abbreviation of this phone number's location. Available for only phone numbers from the US and Canada. | [optional]
**postal_code** | Option<**String**> | The postal or ZIP code of this phone number's location. Available for only phone numbers from the US and Canada. | [optional]
**iso_country** | Option<**String**> | The [ISO country code](https://en.wikipedia.org/wiki/ISO_3166-1_alpha-2) of this phone number. | [optional]
**address_requirements** | Option<**String**> | The type of [Address](https://www.twilio.com/docs/usage/api/address) resource the phone number requires. Can be: `none`, `any`, `local`, or `foreign`. `none` means no address is required. `any` means an address is required, but it can be anywhere in the world. `local` means an address in the phone number's country is required. `foreign` means an address outside of the phone number's country is required. | [optional]
**beta** | Option<**bool**> | Whether the phone number is new to the Twilio platform. Can be: `true` or `false`. | [optional]
**capabilities** | Option<[**models::ApiV2010AccountAvailablePhoneNumberCountryAvailablePhoneNumberLocalCapabilities**](ApiV2010AccountAvailablePhoneNumberCountryAvailablePhoneNumberLocalCapabilities.md)> |  | [optional]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


