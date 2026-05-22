# ApiV2010AccountAddress

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**account_sid** | Option<**String**> | The SID of the [Account](https://www.twilio.com/docs/iam/api/account) that is responsible for the Address resource. | [optional]
**city** | Option<**String**> | The city in which the address is located. | [optional]
**customer_name** | Option<**String**> | The name associated with the address.This property has a maximum length of 16 4-byte characters, or 21 3-byte characters. | [optional]
**date_created** | Option<**String**> | The date and time in GMT that the resource was created specified in [RFC 2822](https://www.ietf.org/rfc/rfc2822.txt) format. | [optional]
**date_updated** | Option<**String**> | The date and time in GMT that the resource was last updated specified in [RFC 2822](https://www.ietf.org/rfc/rfc2822.txt) format. | [optional]
**friendly_name** | Option<**String**> | The string that you assigned to describe the resource. | [optional]
**iso_country** | Option<**String**> | The ISO country code of the address. | [optional]
**postal_code** | Option<**String**> | The postal code of the address. | [optional]
**region** | Option<**String**> | The state or region of the address. | [optional]
**sid** | Option<**String**> | The unique string that that we created to identify the Address resource. | [optional]
**street** | Option<**String**> | The number and street address of the address. | [optional]
**uri** | Option<**String**> | The URI of the resource, relative to `https://api.twilio.com`. | [optional]
**emergency_enabled** | Option<**bool**> | Whether emergency calling has been enabled on this number. | [optional]
**validated** | Option<**bool**> | Whether the address has been validated to comply with local regulation. In countries that require valid addresses, an invalid address will not be accepted. `true` indicates the Address has been validated. `false` indicate the country doesn't require validation or the Address is not valid. | [optional]
**verified** | Option<**bool**> | Whether the address has been verified to comply with regulation. In countries that require valid addresses, an invalid address will not be accepted. `true` indicates the Address has been verified. `false` indicate the country doesn't require verified or the Address is not valid. | [optional]
**street_secondary** | Option<**String**> | The additional number and street address of the address. | [optional]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


