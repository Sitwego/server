# \Api20100401LocalApi

All URIs are relative to *https://api.twilio.com*

Method | HTTP request | Description
------------- | ------------- | -------------
[**list_available_phone_number_local**](Api20100401LocalApi.md#list_available_phone_number_local) | **GET** /2010-04-01/Accounts/{AccountSid}/AvailablePhoneNumbers/{CountryCode}/Local.json | 



## list_available_phone_number_local

> models::ListAvailablePhoneNumberLocalResponse list_available_phone_number_local(account_sid, country_code, area_code, contains, sms_enabled, mms_enabled, voice_enabled, exclude_all_address_required, exclude_local_address_required, exclude_foreign_address_required, beta, near_number, near_lat_long, distance, in_postal_code, in_region, in_rate_center, in_lata, in_locality, fax_enabled, page_size, page, page_token)




### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**account_sid** | **String** | The SID of the [Account](https://www.twilio.com/docs/iam/api/account) requesting the AvailablePhoneNumber resources. | [required] |
**country_code** | **String** | The [ISO-3166-1](https://en.wikipedia.org/wiki/ISO_3166-1_alpha-2) country code of the country from which to read phone numbers. | [required] |
**area_code** | Option<**i32**> | The area code of the phone numbers to read. Applies to only phone numbers in the US and Canada. |  |
**contains** | Option<**String**> | Matching pattern to identify phone numbers. This pattern can be between 2 and 16 characters long and allows all digits (0-9) and all non-diacritic latin alphabet letters (a-z, A-Z). It accepts four meta-characters: `*`, `%`, `+`, `$`. The `*` and `%` meta-characters can appear multiple times in the pattern. To match wildcards at the beginning or end of the pattern, use `*` to match any single character or `%` to match a sequence of characters. If you use the wildcard patterns, it must include at least two non-meta-characters, and wildcards cannot be used between non-meta-characters. To match the beginning of a pattern, start the pattern with `+`. To match the end of the pattern, append the pattern with `$`. These meta-characters can't be adjacent to each other. |  |
**sms_enabled** | Option<**bool**> | Whether the phone numbers can receive text messages. Can be: `true` or `false`. |  |
**mms_enabled** | Option<**bool**> | Whether the phone numbers can receive MMS messages. Can be: `true` or `false`. |  |
**voice_enabled** | Option<**bool**> | Whether the phone numbers can receive calls. Can be: `true` or `false`. |  |
**exclude_all_address_required** | Option<**bool**> | Whether to exclude phone numbers that require an [Address](https://www.twilio.com/docs/usage/api/address). Can be: `true` or `false` and the default is `false`. |  |
**exclude_local_address_required** | Option<**bool**> | Whether to exclude phone numbers that require a local [Address](https://www.twilio.com/docs/usage/api/address). Can be: `true` or `false` and the default is `false`. |  |
**exclude_foreign_address_required** | Option<**bool**> | Whether to exclude phone numbers that require a foreign [Address](https://www.twilio.com/docs/usage/api/address). Can be: `true` or `false` and the default is `false`. |  |
**beta** | Option<**bool**> | Whether to read phone numbers that are new to the Twilio platform. Can be: `true` or `false` and the default is `true`. |  |
**near_number** | Option<**String**> | Given a phone number, find a geographically close number within `distance` miles. Distance defaults to 25 miles. Applies to only phone numbers in the US and Canada. |  |
**near_lat_long** | Option<**String**> | Given a latitude/longitude pair `lat,long` find geographically close numbers within `distance` miles. Applies to only phone numbers in the US and Canada. |  |
**distance** | Option<**i32**> | The search radius, in miles, for a `near_` query.  Can be up to `500` and the default is `25`. Applies to only phone numbers in the US and Canada. |  |
**in_postal_code** | Option<**String**> | Limit results to a particular postal code. Given a phone number, search within the same postal code as that number. Applies to only phone numbers in the US and Canada. |  |
**in_region** | Option<**String**> | Limit results to a particular region, state, or province. Given a phone number, search within the same region as that number. Applies to only phone numbers in the US and Canada. |  |
**in_rate_center** | Option<**String**> | Limit results to a specific rate center, or given a phone number search within the same rate center as that number. Requires `in_lata` to be set as well. Applies to only phone numbers in the US and Canada. |  |
**in_lata** | Option<**String**> | Limit results to a specific local access and transport area ([LATA](https://en.wikipedia.org/wiki/Local_access_and_transport_area)). Given a phone number, search within the same [LATA](https://en.wikipedia.org/wiki/Local_access_and_transport_area) as that number. Applies to only phone numbers in the US and Canada. |  |
**in_locality** | Option<**String**> | Limit results to a particular locality or city. Given a phone number, search within the same Locality as that number. |  |
**fax_enabled** | Option<**bool**> | Whether the phone numbers can receive faxes. Can be: `true` or `false`. |  |
**page_size** | Option<**i64**> | How many resources to return in each list page. The default is 50, and the maximum is 1000. |  |
**page** | Option<**i32**> | The page index. This value is simply for client state. |  |
**page_token** | Option<**String**> | The page token. This is provided by the API. |  |

### Return type

[**models::ListAvailablePhoneNumberLocalResponse**](ListAvailablePhoneNumberLocalResponse.md)

### Authorization

[accountSid_authToken](../README.md#accountSid_authToken)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

