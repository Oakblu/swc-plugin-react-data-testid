// fieldset as root wrapper; inputs rendered via .map() with index param
const OtpInput = ({ value = '', length = 6 }: {
    value?: string;
    length?: number;
})=>{
    const chars = Array.from({
        length
    }, (_, i)=>value[i] ?? '');
    return <fieldset data-testid="OtpInput.fieldset" className="flex gap-2 border-none p-0 m-0">
      {chars.map((char, index)=><input data-testid={"OtpInput.input-" + index} key={`otp-digit-${index}`} type="text" value={char} aria-label={`Digit ${index + 1}`}/>)}
    </fieldset>;
};
